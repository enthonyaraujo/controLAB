//! Scientific Rust engine. The UI remains on the Python engine until parity gates pass.

pub mod controller_math;
pub mod controllers;
pub mod dispatch;
mod exact_transfer;
pub mod execution;
pub mod formulas;
pub mod frequency_response;
pub mod input;
pub mod lgr;
pub mod localization;
pub mod polynomial;
pub mod presentation;
pub mod resources;
pub mod routh;
pub mod simulation;
pub mod state_space;
pub mod symbolic;
pub mod time_response;
pub mod transfer;
pub mod updates;
pub mod values;

pub type Result<T> = std::result::Result<T, String>;
