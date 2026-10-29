//! # blocklab-core
//!
//! Motor do BlockLab, sem interface gráfica:
//!
//! * [`algebra`] — polinômios e funções racionais exatas, com MDC;
//! * [`parser`] — texto como `10/s(s+1)` → função racional;
//! * [`diagram`] — modelo do diagrama de blocos (JSON) e validação;
//! * [`sfg`] — grafo de fluxo de sinal e fórmula de Mason.
//!
//! Uso típico:
//!
//! ```
//! use blocklab_core::{diagram::examples, solve};
//!
//! let report = solve(&examples::series()).unwrap();
//! assert_eq!(report.transfer.to_string(), "G1*G2");
//! ```

pub mod algebra;
pub mod diagram;
pub mod format;
pub mod parser;
pub mod sfg;

pub use sfg::{MasonReport, SolveError, solve, solve_with};
