use displaydoc::Display;
use thiserror::Error;

pub(crate) type ABIResult<T, E = ABIError> = core::result::Result<T, E>;

#[derive(Display, Error, Debug)]
pub enum ABIError {
    /// Runtime error: {0}
    Error(#[from] anyhow::Error),
    /// Wasmer runtime error: {0}
    RuntimeError(#[from] wasmer::RuntimeError),
    /// Serde error: {0}
    SerdeError(#[from] serde_json::Error),
}

macro_rules! abi_bail {
    ($err:expr) => {
        return Err(crate::as_execution::ABIError::Error(anyhow::anyhow!(
            $err.to_string()
        )))
    };
}

pub(crate) use abi_bail;
