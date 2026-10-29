//! Desempenho e cancelamento.
//!
//! O limite de 100 ms vale para build **release**:
//!
//! ```text
//! cargo test --release --test performance -- --nocapture
//! ```
//!
//! Em debug (sem otimização) o mesmo código é ~10–30× mais lento, então
//! o limite é mais folgado — o teste ainda pega regressões grosseiras.

use std::time::{Duration, Instant};

use blocklab_core::algebra::CancelToken;
use blocklab_core::diagram::{Diagram, NodeId, PortRef, Pos, Side, Sign};
use blocklab_core::format::{Mode, pretty};
use blocklab_core::parser::parse_rational;
use blocklab_core::{SolveError, solve, solve_with};

const LIMIT: Duration = if cfg!(debug_assertions) {
    Duration::from_millis(3_000)
} else {
    Duration::from_millis(100)
};

fn at(x: f32) -> Pos {
    Pos::new(x, 0.0)
}

/// Um estágio com realimentação local: entra em `input`, devolve a saída.
///   input → Σ(+, −H) → G → ramificação → (saída, H de volta)
fn stage(d: &mut Diagram, input: NodeId, input_out: u8, g: &str, h: &str) -> (NodeId, u8) {
    let sum = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        at(0.0),
    );
    let gb = d.add_block(g, at(0.0));
    let br = d.add_branch(2, at(0.0));
    let hb = d.add_block(h, at(0.0));
    d.connect(PortRef::new(input, input_out), PortRef::new(sum, 0))
        .unwrap();
    d.link(sum, gb, 0).unwrap();
    d.link(gb, br, 0).unwrap();
    d.connect(PortRef::new(br, 1), PortRef::new(hb, 0)).unwrap();
    d.link(hb, sum, 1).unwrap();
    (br, 0)
}

/// `n` estágios em cascata (laços locais mutuamente disjuntos), um laço
/// externo H0 da saída até a entrada (toca todos) e um caminho direto
/// extra F que pula os estágios do meio.
fn cascade(gains: &[(&str, &str)], h0: &str, f: &str) -> Diagram {
    let mut d = Diagram::new();
    let x = d.add_input("X", at(0.0));
    let outer = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        at(0.0),
    );
    d.link(x, outer, 0).unwrap();
    // Ramifica logo após o somador externo: um ramo vai aos estágios, o
    // outro alimenta o caminho direto F.
    let split = d.add_branch(2, at(0.0));
    d.link(outer, split, 0).unwrap();
    let mut cur = (split, 0u8);
    for (g, h) in gains {
        cur = stage(&mut d, cur.0, cur.1, g, h);
    }
    let fb = d.add_block(f, at(0.0));
    d.connect(PortRef::new(split, 1), PortRef::new(fb, 0))
        .unwrap();
    let join = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Plus, Side::Top)],
        at(0.0),
    );
    d.connect(PortRef::new(cur.0, cur.1), PortRef::new(join, 0))
        .unwrap();
    d.link(fb, join, 1).unwrap();
    let yb = d.add_branch(2, at(0.0));
    d.link(join, yb, 0).unwrap();
    let y = d.add_output("Y", at(0.0));
    d.connect(PortRef::new(yb, 0), PortRef::new(y, 0)).unwrap();
    let h0b = d.add_block(h0, at(0.0));
    d.connect(PortRef::new(yb, 1), PortRef::new(h0b, 0))
        .unwrap();
    d.link(h0b, outer, 1).unwrap();
    d
}

fn timed(name: &str, d: &Diagram) -> Duration {
    // Melhor de 3, para não medir ruído do sistema.
    let mut best = Duration::MAX;
    let mut report = None;
    for _ in 0..3 {
        let t = Instant::now();
        report = Some(solve(d).unwrap());
        best = best.min(t.elapsed());
    }
    let r = report.unwrap();
    // A formatação (simbólica e numérica, texto e LaTeX) entra na conta:
    // é o que a interface faz depois de cada cálculo.
    let t = Instant::now();
    for mode in [Mode::Symbolic, Mode::Numeric] {
        let p = pretty(&r.transfer, mode);
        let _ = (p.text(), p.latex());
    }
    let formatting = t.elapsed();
    println!(
        "{name}: {} laços, {} combinações disjuntas, {} caminhos — Mason {:?}, formatação {:?}",
        r.loops.len(),
        r.nontouching.len(),
        r.paths.len(),
        best,
        formatting
    );
    let best = best + formatting;
    assert!(r.loops.len() >= 4);
    assert!(!r.nontouching.is_empty());
    best
}

#[test]
fn quatro_lacos_simbolico() {
    let d = cascade(&[("G1", "H1"), ("G2", "H2"), ("G3", "H3")], "H0", "G5");
    let elapsed = timed("simbólico", &d);
    assert!(elapsed < LIMIT, "{elapsed:?} ≥ {LIMIT:?}");

    // Confere o resultado: com F em paralelo aos estágios, a fórmula é
    // T = (P + F) / (1 + (P + F)·H0), P = Π Gi/(1 + Gi·Hi).
    let p = "G1/(1+G1 H1) * G2/(1+G2 H2) * G3/(1+G3 H3)";
    let expected = parse_rational(&format!("({p} + G5) / (1 + ({p} + G5) H0)")).unwrap();
    assert_eq!(solve(&d).unwrap().transfer, expected);
}

#[test]
fn quatro_lacos_com_s_e_parametros() {
    let d = cascade(
        &[
            ("K/(s+a)", "0.5"),
            ("10/(s(s+2))", "s"),
            ("(s+3)/(s^2+4s+5)", "2/(s+1)"),
        ],
        "1/(T s + 1)",
        "Kd s",
    );
    let elapsed = timed("com s e parâmetros", &d);
    assert!(elapsed < LIMIT, "{elapsed:?} ≥ {LIMIT:?}");
}

#[test]
fn quatro_lacos_numerico() {
    let d = cascade(
        &[
            ("2/(s+1)", "0.5"),
            ("10/(s(s+2))", "s"),
            ("(s+3)/(s^2+4s+5)", "2/(s+1)"),
        ],
        "1/(0.1s + 1)",
        "0.2",
    );
    let elapsed = timed("numérico", &d);
    assert!(elapsed < LIMIT, "{elapsed:?} ≥ {LIMIT:?}");
}

#[test]
fn cancelado_antes_de_comecar() {
    let d = cascade(&[("G1", "H1"), ("G2", "H2")], "H0", "G5");
    let token = CancelToken::new();
    token.cancel();
    assert_eq!(solve_with(&d, &token).unwrap_err(), SolveError::Cancelled);
}

#[test]
fn cancelamento_por_outra_thread_responde_rapido() {
    // Diagrama grande o bastante para levar bem mais que alguns ms:
    // 10 estágios com termos em s e símbolos ⇒ 2^10 combinações de laços.
    let gains: Vec<(String, String)> = (1..=10)
        .map(|i| (format!("K{i}/(s + a{i})"), format!("(s + b{i})/(s + c{i})")))
        .collect();
    let refs: Vec<(&str, &str)> = gains
        .iter()
        .map(|(g, h)| (g.as_str(), h.as_str()))
        .collect();
    let d = cascade(&refs, "H0", "F");

    let token = CancelToken::new();
    let remote = token.clone();
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        let t = Instant::now();
        remote.cancel();
        t
    });
    let result = solve_with(&d, &token);
    let finished = Instant::now();
    let cancelled_at = canceller.join().unwrap();
    match result {
        Err(SolveError::Cancelled) => {
            let reaction = finished.duration_since(cancelled_at);
            println!("cancelamento atendido em {reaction:?}");
            assert!(reaction < Duration::from_millis(200), "{reaction:?}");
        }
        // Se a máquina for rápida a ponto de terminar em 20 ms, tudo bem.
        Ok(_) => println!("terminou antes do cancelamento"),
        Err(e) => panic!("{e}"),
    }
}
