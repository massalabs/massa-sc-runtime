use displaydoc::Display;
use thiserror::Error;

#[derive(Error, Debug, Clone, Display)]
pub enum WasmV1Error {
    /// Instanciation error: {0}
    InstanciationError(String),
}
