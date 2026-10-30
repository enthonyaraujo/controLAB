//! Cálculo em segundo plano, para a janela nunca travar.
//!
//! Cada pedido roda numa thread própria com um [`CancelToken`]. Um pedido
//! novo cancela o anterior (que desiste sozinho em poucos ms) e ganha uma
//! *geração* maior; resultados de gerações antigas que ainda cheguem pelo
//! canal são descartados.
//!
//! Só recalcula quando muda algo que afeta a matemática: mover um nó ou
//! virar um bloco não dispara cálculo (ver [`semantic_key`]).

use std::sync::mpsc::{Receiver, Sender, channel};

use blocklab_core::algebra::CancelToken;
use blocklab_core::diagram::{Diagram, Pos};
use blocklab_core::{MasonReport, SolveError, solve_with};

pub type Outcome = Result<MasonReport, SolveError>;

pub struct Solver {
    tx: Sender<(u64, Outcome)>,
    rx: Receiver<(u64, Outcome)>,
    generation: u64,
    running: Option<CancelToken>,
    last_key: Option<String>,
}

impl Default for Solver {
    fn default() -> Self {
        let (tx, rx) = channel();
        Solver {
            tx,
            rx,
            generation: 0,
            running: None,
            last_key: None,
        }
    }
}

/// O que importa para o cálculo simbólico: tipos, expressões, sinais e
/// ligações. Posições, espelhamento e parâmetros numéricos ficam de fora.
pub fn semantic_key(d: &Diagram) -> String {
    let mut c = d.clone();
    for n in &mut c.nodes {
        n.pos = Pos::default();
        n.flipped = false;
    }
    c.params.clear();
    c.to_json()
}

impl Solver {
    /// Pede o cálculo de `d` se ele mudou desde o último pedido.
    /// `on_done` é chamado da thread de trabalho ao terminar (a interface
    /// usa para pedir um redesenho).
    pub fn request(&mut self, d: &Diagram, on_done: impl Fn() + Send + 'static) {
        let key = semantic_key(d);
        if self.last_key.as_ref() == Some(&key) {
            return;
        }
        self.last_key = Some(key);
        self.cancel();
        self.generation += 1;
        let generation = self.generation;
        let token = CancelToken::new();
        self.running = Some(token.clone());
        let tx = self.tx.clone();
        let d = d.clone();
        std::thread::spawn(move || {
            let outcome = solve_with(&d, &token);
            // O receptor pode já ter sido descartado (janela fechada).
            let _ = tx.send((generation, outcome));
            on_done();
        });
    }

    /// Cancela o cálculo em andamento (se houver).
    pub fn cancel(&mut self) {
        if let Some(t) = self.running.take() {
            t.cancel();
        }
    }

    /// Esquece o último pedido: o próximo `request` recalcula mesmo que o
    /// diagrama não tenha mudado (usado pelo botão "recalcular" após
    /// cancelar).
    pub fn invalidate(&mut self) {
        self.last_key = None;
    }

    pub fn is_running(&self) -> bool {
        self.running.is_some()
    }

    /// Resultado mais recente, se chegou. Resultados de gerações velhas
    /// são ignorados.
    pub fn poll(&mut self) -> Option<Outcome> {
        let mut latest = None;
        while let Ok((generation, outcome)) = self.rx.try_recv() {
            if generation == self.generation {
                self.running = None;
                latest = Some(outcome);
            }
        }
        latest
    }
}

impl Drop for Solver {
    fn drop(&mut self) {
        if let Some(token) = &self.running {
            token.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blocklab_core::diagram::{Sign, examples};
    use std::time::{Duration, Instant};

    fn wait(s: &mut Solver) -> Outcome {
        let t = Instant::now();
        loop {
            if let Some(o) = s.poll() {
                return o;
            }
            assert!(
                t.elapsed() < Duration::from_secs(10),
                "solver não respondeu"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn calcula_em_segundo_plano() {
        let mut s = Solver::default();
        s.request(&examples::series(), || {});
        assert_eq!(wait(&mut s).unwrap().transfer.to_string(), "G1*G2");
        assert!(!s.is_running());
    }

    #[test]
    fn mover_no_nao_recalcula() {
        let mut s = Solver::default();
        let mut d = examples::feedback(Sign::Minus);
        s.request(&d, || {});
        wait(&mut s).unwrap();
        d.nodes[0].pos.x += 100.0;
        d.nodes[2].flipped = !d.nodes[2].flipped;
        s.request(&d, || {});
        assert!(!s.is_running());
    }

    #[test]
    fn pedido_novo_descarta_o_antigo() {
        let mut s = Solver::default();
        s.request(&examples::series(), || {});
        s.request(&examples::parallel(), || {});
        assert_eq!(wait(&mut s).unwrap().transfer.to_string(), "G1 + G2");
    }
}
