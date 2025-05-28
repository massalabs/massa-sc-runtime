use displaydoc::Display;
use thiserror::Error;

pub type VMResult<T> = Result<T, VMError>;

#[derive(Clone, Error, Display, Debug)]
pub enum VMError {
    /// VM instance error: {0}
    InstanceError(String),
    /// VM execution error: {error}
    ExecutionError { error: String, init_gas_cost: u64 },
    /// Depth error: {0}
    DepthError(String),
}

impl From<ABIError> for VMError {
    fn from(e: ABIError) -> Self {
        match e {
            ABIError::SerdeError(e) => VMError::InstanceError(e),
            ABIError::VMError(e) => VMError::InstanceError(e),
            ABIError::Error(e) => VMError::InstanceError(e.to_string()),
            ABIError::RuntimeError(e) => VMError::InstanceError(e),
            ABIError::DepthError(e) => VMError::DepthError(e),
        }
    }
}

impl From<wasmer::RuntimeError> for VMError {
    fn from(e: wasmer::RuntimeError) -> Self {
        if let Some(err) = e.downcast_ref::<ABIError>() {
            VMError::DepthError(err.to_string())
        } else {
            VMError::InstanceError(e.to_string())
        }
    }
}

impl From<wasmer::ExportError> for VMError {
    fn from(e: wasmer::ExportError) -> Self {
        VMError::InstanceError(e.to_string())
    }
}

impl From<wasmer::InstantiationError> for VMError {
    fn from(e: wasmer::InstantiationError) -> Self {
        VMError::InstanceError(e.to_string())
    }
}

impl From<anyhow::Error> for VMError {
    fn from(value: anyhow::Error) -> Self {
        Self::InstanceError(value.to_string())
    }
}

macro_rules! exec_bail {
    ($err:expr, $init_gas_cost:expr) => {
        return Err(crate::VMError::ExecutionError {
            error: $err.to_string(),
            init_gas_cost: $init_gas_cost,
        })
    };
}

macro_rules! vm_bail {
    ($err:expr) => {
        return Err(crate::VMError::InstanceError($err.to_string()))
    };
}

pub(crate) use exec_bail;
pub(crate) use vm_bail;

use crate::as_execution::ABIError;
