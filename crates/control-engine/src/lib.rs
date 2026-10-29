//! Scientific Rust engine. The UI remains on the Python engine until parity gates pass.

pub mod polynomial;
pub mod transfer;
pub mod simulation;
pub mod presentation;
pub mod values;
pub mod input;
pub mod controller_math;
pub mod time_response;
pub mod frequency_response;
pub mod symbolic;
pub mod routh;
pub mod dispatch;
pub mod state_space;
mod exact_transfer;
pub mod lgr;
pub mod formulas;
pub mod localization;
pub mod controllers;

pub type Result<T> = std::result::Result<T, String>;
