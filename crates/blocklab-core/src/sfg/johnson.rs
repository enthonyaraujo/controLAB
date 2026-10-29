//! Todos os laços (ciclos simples) de um grafo dirigido — algoritmo de
//! Johnson (1975).
//!
//! Ideia geral: para cada nó `s`, em ordem crescente, procuramos os
//! ciclos cujo **menor** nó é `s`. Assim cada ciclo é achado exatamente
//! uma vez. Para isso olhamos só o subgrafo dos nós ≥ s, e dentro dele só
//! a componente fortemente conexa de `s` (os nós que vão até `s` e voltam
//! dele) — fora dela não existe ciclo passando por `s`.
//!
//! A busca em profundidade a partir de `s` usaria tempo exponencial
//! revisitando becos sem saída. O truque de Johnson é **bloquear** um nó
//! quando se descobre que dele não se chega a `s` sem repetir o caminho
//! atual, e só desbloqueá-lo quando algo nesse caminho mudar (a lista
//! `b` guarda quem depende de quem). Isso dá tempo O((n + e)(c + 1)),
//! proporcional ao número `c` de ciclos.

use std::collections::VecDeque;

use super::SolveError;
use crate::algebra::CancelToken;

/// Ciclos simples; cada um começa no seu menor nó e não repete o
/// primeiro nó no fim. Um laço próprio (v → v) aparece como `[v]`.
pub fn simple_cycles(
    succ: &[Vec<usize>],
    limit: usize,
    cancel: &CancelToken,
) -> Result<Vec<Vec<usize>>, SolveError> {
    let n = succ.len();
    let mut st = State {
        succ,
        in_comp: vec![false; n],
        blocked: vec![false; n],
        b: vec![Vec::new(); n],
        stack: Vec::new(),
        cycles: Vec::new(),
        limit,
        cancel,
    };
    for s in 0..n {
        let comp = strong_component(succ, s);
        let self_loop = succ[s].contains(&s);
        if comp.iter().filter(|&&x| x).count() < 2 && !self_loop {
            continue;
        }
        st.in_comp = comp;
        st.blocked.iter_mut().for_each(|x| *x = false);
        st.b.iter_mut().for_each(Vec::clear);
        st.circuit(s, s)?;
    }
    Ok(st.cycles)
}

struct State<'a> {
    succ: &'a [Vec<usize>],
    in_comp: Vec<bool>,
    blocked: Vec<bool>,
    b: Vec<Vec<usize>>,
    stack: Vec<usize>,
    cycles: Vec<Vec<usize>>,
    limit: usize,
    cancel: &'a CancelToken,
}

impl State<'_> {
    /// Procura ciclos que passam pelo caminho atual + `v` e voltam a `s`.
    /// Devolve `true` se achou algum.
    fn circuit(&mut self, v: usize, s: usize) -> Result<bool, SolveError> {
        if self.cancel.is_cancelled() {
            return Err(SolveError::Cancelled);
        }
        let mut found = false;
        self.stack.push(v);
        self.blocked[v] = true;
        for i in 0..self.succ[v].len() {
            let w = self.succ[v][i];
            if !self.in_comp[w] {
                continue;
            }
            if w == s {
                if self.cycles.len() >= self.limit {
                    return Err(SolveError::TooComplex {
                        what: "laços",
                        limit: self.limit,
                    });
                }
                self.cycles.push(self.stack.clone());
                found = true;
            } else if !self.blocked[w] && self.circuit(w, s)? {
                found = true;
            }
        }
        if found {
            // Achou ciclo por aqui: v pode ser reusado por outros caminhos.
            self.unblock(v);
        } else {
            // Beco sem saída por enquanto: v só volta a valer a pena quando
            // algum sucessor dele for desbloqueado.
            for i in 0..self.succ[v].len() {
                let w = self.succ[v][i];
                if self.in_comp[w] && !self.b[w].contains(&v) {
                    self.b[w].push(v);
                }
            }
        }
        self.stack.pop();
        Ok(found)
    }

    fn unblock(&mut self, u: usize) {
        self.blocked[u] = false;
        while let Some(w) = self.b[u].pop() {
            if self.blocked[w] {
                self.unblock(w);
            }
        }
    }
}

/// Componente fortemente conexa de `s` no subgrafo dos nós ≥ s:
/// nós alcançáveis a partir de `s` **e** que alcançam `s`.
fn strong_component(succ: &[Vec<usize>], s: usize) -> Vec<bool> {
    let n = succ.len();
    let mut pred = vec![Vec::new(); n];
    for (v, ws) in succ.iter().enumerate() {
        for &w in ws {
            pred[w].push(v);
        }
    }
    let bfs = |adj: &[Vec<usize>]| {
        let mut seen = vec![false; n];
        seen[s] = true;
        let mut queue = VecDeque::from([s]);
        while let Some(v) = queue.pop_front() {
            for &w in &adj[v] {
                if w >= s && !seen[w] {
                    seen[w] = true;
                    queue.push_back(w);
                }
            }
        }
        seen
    };
    let (fwd, bwd) = (bfs(succ), bfs(&pred));
    fwd.iter().zip(&bwd).map(|(a, b)| *a && *b).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cycles(succ: &[Vec<usize>]) -> Vec<Vec<usize>> {
        let mut c = simple_cycles(succ, 1000, &CancelToken::new()).unwrap();
        c.sort();
        c
    }

    #[test]
    fn triangulo_com_atalho_e_laco_proprio() {
        // 0→1→2→0, 1→0, 2→2
        let succ = vec![vec![1], vec![0, 2], vec![0, 2]];
        assert_eq!(cycles(&succ), vec![vec![0, 1], vec![0, 1, 2], vec![2]]);
    }

    #[test]
    fn grafo_completo_k4_tem_20_ciclos() {
        // Em K4 dirigido: C(4,2)·1 + C(4,3)·2 + C(4,4)·6 = 6 + 8 + 6 = 20.
        let succ: Vec<Vec<usize>> = (0..4)
            .map(|v| (0..4).filter(|&w| w != v).collect())
            .collect();
        assert_eq!(cycles(&succ).len(), 20);
    }

    #[test]
    fn sem_ciclos() {
        let succ = vec![vec![1, 2], vec![2], vec![]];
        assert!(cycles(&succ).is_empty());
    }

    #[test]
    fn limite_de_ciclos() {
        let succ: Vec<Vec<usize>> = (0..4)
            .map(|v| (0..4).filter(|&w| w != v).collect())
            .collect();
        let err = simple_cycles(&succ, 5, &CancelToken::new()).unwrap_err();
        assert!(matches!(err, SolveError::TooComplex { limit: 5, .. }));
    }
}
