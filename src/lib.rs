mod as_execution;
mod error;
mod execution;
mod middlewares;
mod settings;
mod tunable_memory;
mod types;
mod wasmv1_execution;

pub use error::VMError;
pub use execution::{run_function, run_main};
pub use execution::{Compiler, RuntimeModule};
pub use types::*;

pub fn print_feats() {
    #[cfg(test)]
    println!("Massa-sc-runtime print_feats: cfg(test) is ENABLED");
    #[cfg(not(test))]
    println!("Massa-sc-runtime print_feats: cfg(test) is DISABLED");
    #[cfg(feature = "gas_calibration")]
    println!("Massa-sc-runtime print_feats: cfg(gas_calibration) is ENABLED");
    #[cfg(not(feature = "gas_calibration"))]
    println!("Massa-sc-runtime print_feats: cfg(gas_calibration) is DISABLED");
    #[cfg(feature = "testing")]
    println!("Massa-sc-runtime print_feats: cfg(testing) is ENABLED");
    #[cfg(not(feature = "testing"))]
    println!("Massa-sc-runtime: cfg(testing) is DISABLED");
}

#[cfg(feature = "gas_calibration")]
pub use execution::run_main_gc;
#[cfg(feature = "gas_calibration")]
pub use middlewares::gas_calibration::GasCalibrationResult;

#[cfg(test)]
mod tests;
