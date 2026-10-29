//! Casos clássicos de diagrama de blocos resolvidos pela fórmula de
//! Mason, comparados com o resultado conhecido (escrito como texto e lido
//! pelo parser, para ficar legível).

use blocklab_core::algebra::RationalFn;
use blocklab_core::diagram::{
    ConnectError, Diagram, DiagramError, NodeKind, ParamError, PortDir, PortRef, Pos, Side, Sign,
    examples, validate,
};
use blocklab_core::parser::{ParseErrorKind, parse_rational};
use blocklab_core::{SolveError, solve};

fn expr(src: &str) -> RationalFn {
    parse_rational(src).unwrap()
}

fn transfer(d: &Diagram) -> RationalFn {
    solve(d).unwrap_or_else(|e| panic!("{e}: {e:?}")).transfer
}

fn at(x: f32) -> Pos {
    Pos::new(x, 0.0)
}

/// X → bloco(expr) → Y
fn single_block(e: &str) -> Diagram {
    let mut d = Diagram::new();
    let x = d.add_input("X", at(0.0));
    let b = d.add_block(e, at(100.0));
    let y = d.add_output("Y", at(200.0));
    d.link(x, b, 0).unwrap();
    d.link(b, y, 0).unwrap();
    d
}

#[test]
fn serie() {
    assert_eq!(transfer(&examples::series()), expr("G1 G2"));
}

#[test]
fn paralelo() {
    assert_eq!(transfer(&examples::parallel()), expr("G1 + G2"));
}

#[test]
fn realimentacao_negativa() {
    assert_eq!(
        transfer(&examples::feedback(Sign::Minus)),
        expr("G/(1 + G H)")
    );
}

#[test]
fn realimentacao_positiva() {
    assert_eq!(
        transfer(&examples::feedback(Sign::Plus)),
        expr("G/(1 - G H)")
    );
}

#[test]
fn realimentacao_numerica() {
    let d = examples::feedback_with("10/(s(s+1))", "1", Sign::Minus);
    assert_eq!(transfer(&d), expr("10/(s^2 + s + 10)"));
}

#[test]
fn bloco_com_fator_comum() {
    assert_eq!(
        transfer(&single_block("(s+1)/((s+1)(s+2))")),
        expr("1/(s+2)")
    );
}

#[test]
fn realimentacao_unitaria_simbolica() {
    let d = examples::feedback_with("K/(s+a)", "1", Sign::Minus);
    assert_eq!(transfer(&d), expr("K/(s + a + K)"));
}

#[test]
fn dois_lacos_que_nao_se_tocam() {
    let r = solve(&examples::two_loops()).unwrap();
    assert_eq!(r.loops.len(), 2);
    assert_eq!(r.nontouching, vec![vec![0, 1]]);
    // Δ = 1 − (L1 + L2) + L1·L2, com L1 = −G1H1 e L2 = −G2H2.
    assert_eq!(r.delta, expr("1 + G1 H1 + G2 H2 + G1 H1 G2 H2"));
    assert_eq!(r.transfer, expr("G1 G2 / ((1 + G1 H1)(1 + G2 H2))"));
}

#[test]
fn lacos_aninhados() {
    let r = solve(&examples::nested_loops()).unwrap();
    assert_eq!(r.loops.len(), 2);
    assert!(r.nontouching.is_empty());
    assert_eq!(r.transfer, expr("G1 G2 / (1 + G2 H1 + G1 G2 H2)"));
}

#[test]
fn entrada_ligada_direto_na_saida() {
    let mut d = Diagram::new();
    let x = d.add_input("X", at(0.0));
    let y = d.add_output("Y", at(100.0));
    d.link(x, y, 0).unwrap();
    assert!(transfer(&d).is_one());
}

#[test]
fn saida_com_varios_fios_equivale_a_ramificacao() {
    // Mesma realimentação negativa, mas a saída de G alimenta Y e H
    // direto, sem nó de ramificação.
    let mut d = Diagram::new();
    let x = d.add_input("X", at(0.0));
    let sum = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        at(1.0),
    );
    let g = d.add_block("G", at(2.0));
    let h = d.add_block("H", at(3.0));
    let y = d.add_output("Y", at(4.0));
    d.link(x, sum, 0).unwrap();
    d.link(sum, g, 0).unwrap();
    d.link(g, y, 0).unwrap();
    d.link(g, h, 0).unwrap();
    d.link(h, sum, 1).unwrap();
    assert_eq!(transfer(&d), expr("G/(1 + G H)"));
}

#[test]
fn laco_algebrico_de_ganho_um() {
    // X → Σ(+, +) → ramificação → Y, e a ramificação volta para o Σ sem
    // nenhum bloco: x = X + x não tem solução. Δ = 1 − 1 = 0.
    let mut d = Diagram::new();
    let x = d.add_input("X", at(0.0));
    let sum = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Plus, Side::Bottom)],
        at(1.0),
    );
    let br = d.add_branch(2, at(2.0));
    let y = d.add_output("Y", at(3.0));
    d.link(x, sum, 0).unwrap();
    d.link(sum, br, 0).unwrap();
    d.connect(PortRef::new(br, 0), PortRef::new(y, 0)).unwrap();
    d.connect(PortRef::new(br, 1), PortRef::new(sum, 1))
        .unwrap();
    assert_eq!(solve(&d).unwrap_err(), SolveError::DegenerateDelta);
}

#[test]
fn erros_de_validacao_apontam_os_nos() {
    let errs = validate(&Diagram::new()).unwrap_err();
    assert_eq!(errs, vec![DiagramError::NoInput, DiagramError::NoOutput]);

    // X → G, com a saída de G e a entrada de Y soltas.
    let mut d = Diagram::new();
    let x = d.add_input("X", at(0.0));
    let g = d.add_block("G", at(1.0));
    let y = d.add_output("Y", at(2.0));
    d.link(x, g, 0).unwrap();
    let errs = validate(&d).unwrap_err();
    for e in [
        DiagramError::LoosePort {
            node: g,
            port: 0,
            dir: PortDir::Out,
        },
        DiagramError::LoosePort {
            node: y,
            port: 0,
            dir: PortDir::In,
        },
        DiagramError::NoPathToOutput(x),
        DiagramError::NoPathToOutput(g),
        DiagramError::UnreachableFromInput(y),
    ] {
        assert!(errs.contains(&e), "faltou {e:?} em {errs:?}");
    }
    assert_eq!(
        DiagramError::NoPathToOutput(g).describe(&d),
        "bloco 'G' (#1): não há caminho deste nó até a saída"
    );
}

#[test]
fn expressao_invalida_aponta_bloco_e_posicao() {
    let d = single_block("s^a");
    let Err(SolveError::Invalid(errs)) = solve(&d) else {
        panic!("deveria ser inválido");
    };
    let [DiagramError::BadExpr { node, err }] = errs.as_slice() else {
        panic!("{errs:?}");
    };
    assert_eq!(
        d.node(*node).unwrap().kind,
        NodeKind::Block { expr: "s^a".into() }
    );
    assert_eq!(err.kind, ParseErrorKind::SymbolicExponent("a".into()));
    assert_eq!(
        errs[0].describe(&d),
        "bloco 's^a' (#1): expressão inválida: posição 3: \
         o expoente precisa ser um inteiro literal, não o símbolo 'a'"
    );
}

#[test]
fn ligacoes_invalidas() {
    let mut d = Diagram::new();
    let x = d.add_input("X", at(0.0));
    let g = d.add_block("G", at(1.0));
    d.link(x, g, 0).unwrap();
    assert_eq!(
        d.link(x, g, 0),
        Err(ConnectError::InputTaken { node: g, port: 0 })
    );
    assert_eq!(
        d.link(x, g, 1),
        Err(ConnectError::NoSuchPort {
            node: g,
            port: 1,
            dir: PortDir::In
        })
    );
    // Entrada X não tem porta de entrada.
    assert!(d.link(g, x, 0).is_err());
}

#[test]
fn json_ida_e_volta() {
    let d = examples::reference(Sign::Plus);
    let text = d.to_json();
    let mut back = Diagram::from_json(&text).unwrap();
    assert_eq!(back, d);
    // Um nó novo depois de carregar não pode reusar ids existentes.
    let id = back.add_block("G9", at(0.0));
    assert!(d.nodes.iter().all(|n| n.id != id));
    assert!(d.wires.iter().all(|w| w.id.0 != id.0));
}

#[test]
fn json_de_versao_futura_e_recusado_e_sem_versao_e_aceito() {
    let mut v: serde_json::Value = serde_json::from_str(&examples::series().to_json()).unwrap();
    v["format_version"] = 99.into();
    let err = Diagram::from_json(&v.to_string()).unwrap_err();
    assert!(err.to_string().contains("versão 99"), "{err}");

    // Arquivos antigos (sem o campo) são lidos como versão 1.
    v.as_object_mut().unwrap().remove("format_version");
    let d = Diagram::from_json(&v.to_string()).unwrap();
    assert_eq!(d, examples::series());

    let err = Diagram::from_json("{ não é json").unwrap_err();
    assert!(err.to_string().starts_with("JSON inválido"), "{err}");
}

#[test]
fn json_legivel() {
    let mut d = single_block("K/(s+a)");
    d.params.insert("K".into(), "2".into());
    let text = d.to_json();
    assert!(text.contains(r#""format_version": 1"#), "{text}");
    assert!(text.contains(r#""type": "Block""#), "{text}");
    assert!(text.contains(r#""expr": "K/(s+a)""#), "{text}");
}

#[test]
fn modo_numerico_por_parametros() {
    let mut d = examples::feedback_with("K/(s+a)", "1", Sign::Minus);
    d.params.insert("K".into(), "2".into());
    d.params.insert("a".into(), "0.5".into());
    d.params.insert("b".into(), "".into()); // vazio = ignora
    let params = d.numeric_params().unwrap();
    let t = transfer(&d).substitute(&params).unwrap();
    assert_eq!(t, expr("2/(s + 2.5)"));

    d.params.insert("K".into(), "s + 1".into());
    assert!(matches!(
        d.numeric_params(),
        Err(ParamError::NotNumeric { .. })
    ));
    d.params.remove("K");
    d.params.insert("2x".into(), "1".into());
    assert_eq!(d.numeric_params(), Err(ParamError::BadName("2x".into())));
}
