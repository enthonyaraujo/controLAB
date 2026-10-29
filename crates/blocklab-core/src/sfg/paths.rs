//! Caminhos diretos: todos os caminhos simples (sem repetir nó) da
//! entrada até a saída.

use super::SolveError;
use crate::algebra::CancelToken;

/// Busca em profundidade: anda pelos sucessores marcando os nós do
/// caminho atual; ao chegar em `sink`, registra; ao voltar, desmarca
/// (*backtracking*), para que outros caminhos possam reusar o nó.
pub fn forward_paths(
    succ: &[Vec<usize>],
    source: usize,
    sink: usize,
    cancel: &CancelToken,
) -> Result<Vec<Vec<usize>>, SolveError> {
    struct Dfs<'a> {
        succ: &'a [Vec<usize>],
        sink: usize,
        on_path: Vec<bool>,
        path: Vec<usize>,
        found: Vec<Vec<usize>>,
        cancel: &'a CancelToken,
    }
    impl Dfs<'_> {
        fn visit(&mut self, v: usize) -> Result<(), SolveError> {
            if self.cancel.is_cancelled() {
                return Err(SolveError::Cancelled);
            }
            self.path.push(v);
            if v == self.sink {
                self.found.push(self.path.clone());
            } else {
                self.on_path[v] = true;
                for i in 0..self.succ[v].len() {
                    let w = self.succ[v][i];
                    if !self.on_path[w] {
                        self.visit(w)?;
                    }
                }
                self.on_path[v] = false;
            }
            self.path.pop();
            Ok(())
        }
    }
    let mut dfs = Dfs {
        succ,
        sink,
        on_path: vec![false; succ.len()],
        path: Vec::new(),
        found: Vec::new(),
        cancel,
    };
    dfs.visit(source)?;
    Ok(dfs.found)
}
