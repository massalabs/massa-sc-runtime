use displaydoc::Display;
use thiserror::Error;

#[derive(Error, Debug, Clone, Display)]
pub enum WasmV1Error {
    /// Runtime error: {0}
    RuntimeError(String),
    /// Instanciation error: {0}
    InstanciationError(String),
}
