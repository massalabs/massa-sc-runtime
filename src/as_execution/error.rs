use crate::{InterfaceError, VMError};
use displaydoc::Display;
use thiserror::Error;

pub(crate) type ABIResult<T, E = ABIError> = core::result::Result<T, E>;

#[derive(Error, Display, Debug)]
pub enum ABIError {
    /// Runtime error: {0}
    Error(#[from] anyhow::Error),
    /// Wasmer runtime error: {0}
    RuntimeError(String),
    /// Serde error: {0}
    SerdeError(String),
    /// VM error: {0}
    VMError(String),
    /// {0}
    DepthError(String),
}

impl From<VMError> for ABIError {
    fn from(e: VMError) -> Self {
        match e {
            VMError::InstanceError(e) => ABIError::VMError(e),
            VMError::ExecutionError { error, .. } => ABIError::VMError(error),
            VMError::DepthError(e) => ABIError::DepthError(e),
        }
    }
}

impl From<InterfaceError> for ABIError {
    fn from(e: InterfaceError) -> Self {
        match e {
            InterfaceError::SerdeError(e) => ABIError::SerdeError(e.to_string()),
            InterfaceError::DepthError(e) => ABIError::DepthError(e),
            InterfaceError::GasCalibrationError(e) | InterfaceError::GenericError(e) => {
                ABIError::Error(anyhow::Error::msg(e))
            }
            InterfaceError::IoError(e) => ABIError::Error(e.into()),
            InterfaceError::Utf8Error(e) => ABIError::Error(e.into()),
        }
    }
}

impl From<wasmer::RuntimeError> for ABIError {
    fn from(e: wasmer::RuntimeError) -> Self {
        ABIError::RuntimeError(e.to_string())
    }
}

macro_rules! abi_bail {
    ($err:expr) => {
        return Err(super::ABIError::Error(anyhow::anyhow!($err.to_string())))
    };
}

pub(crate) use abi_bail;
