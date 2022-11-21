mod env;
mod execution;
mod execution_impl;
mod settings;
mod tunable_memory;
mod types;

pub use execution_impl::{run_function, run_main};
pub use types::*;

#[cfg(feature = "gas_calibration")]
pub use execution_impl::run_main_gc;
#[cfg(feature = "gas_calibration")]
pub use middlewares::gas_calibration::GasCalibrationResult;

mod middlewares;

#[cfg(test)]
mod tests;
