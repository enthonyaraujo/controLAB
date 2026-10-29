//! Mason contra a resolução direta do sistema linear.
//!
//! Um SFG é o sistema x_j = Σᵢ gᵢⱼ·xᵢ (para todo nó j ≠ fonte), com a
//! fonte valendo 1. Resolver isso por eliminação de Gauss dá a mesma
//! função de transferência que Mason — mas por um caminho totalmente
//! independente (sem caminhos, laços nem combinações). Se o Johnson
//! esquecer um laço, ou o Δ errar um sinal, este teste pega.
//!
//! Cada ramo recebe um símbolo próprio (e0, e1, ...), então nenhum
//! cancelamento "por sorte" esconde um erro.

use blocklab_core::algebra::{CancelToken, RationalFn, Var};
use blocklab_core::sfg::{Sfg, mason};
use proptest::prelude::*;

/// Grafo aleatório: `n` nós, fonte 0, saída n−1; nenhum ramo chega na
/// fonte; laços próprios permitidos.
fn random_sfg() -> impl Strategy<Value = Sfg> {
    (2usize..=5).prop_flat_map(|n| {
        let pairs: Vec<(usize, usize)> = (0..n).flat_map(|a| (1..n).map(move |b| (a, b))).collect();
        proptest::sample::subsequence(pairs.clone(), 0..=pairs.len().min(8)).prop_map(
            move |edges| {
                let mut g = Sfg::default();
                for i in 0..n {
                    g.add_node(format!("n{i}"), None);
                }
                g.source = 0;
                g.sink = n - 1;
                for (k, (a, b)) in edges.into_iter().enumerate() {
                    g.add_edge(a, b, RationalFn::var(Var::named(&format!("e{k}"))));
                }
                g
            },
        )
    })
}

/// Resolve (I − A)·x = b por eliminação de Gauss sobre funções racionais
/// e devolve x[sink]. Incógnitas: todos os nós menos a fonte.
fn gauss(g: &Sfg) -> RationalFn {
    let n = g.nodes.len();
    let unknowns: Vec<usize> = (0..n).filter(|&v| v != g.source).collect();
    let col = |v: usize| unknowns.iter().position(|&u| u == v).unwrap();
    let m = unknowns.len();
    // Linha da equação do nó j: x_j − Σ_{i≠fonte} gᵢⱼ xᵢ = g_{fonte,j}.
    let mut a = vec![vec![RationalFn::zero(); m + 1]; m];
    for (row, &j) in unknowns.iter().enumerate() {
        a[row][row] = RationalFn::one();
        for e in g.edges.iter().filter(|e| e.to == j) {
            if e.from == g.source {
                a[row][m] = &a[row][m] + &e.gain;
            } else {
                let c = col(e.from);
                a[row][c] = &a[row][c] - &e.gain;
            }
        }
    }
    // Escalonamento com pivô = primeiro não nulo da coluna.
    for c in 0..m {
        let p = (c..m)
            .find(|&r| !a[r][c].is_zero())
            .expect("det(I − A) = Δ ≠ 0");
        a.swap(c, p);
        let pivot = a[c][c].clone();
        for x in &mut a[c][c..] {
            *x = &*x / &pivot;
        }
        let pivot_row = a[c].clone();
        for (r, row) in a.iter_mut().enumerate() {
            if r != c && !row[c].is_zero() {
                let f = row[c].clone();
                for (x, p) in row[c..].iter_mut().zip(&pivot_row[c..]) {
                    *x = &*x - &(&f * p);
                }
            }
        }
    }
    a[col(g.sink)][m].clone()
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        timeout: 20_000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn mason_igual_a_gauss(g in random_sfg()) {
        let expected = gauss(&g);
        let report = mason(g, &CancelToken::new()).unwrap();
        prop_assert_eq!(report.transfer, expected);
    }
}
