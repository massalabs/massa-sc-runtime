use displaydoc::Display;
use thiserror::Error;

pub type VMResult<T> = Result<T, VMError>;

#[derive(Clone, Error, Display, Debug)]
pub enum VMError {
    /// VM instance error: {0}
    InstanceError(String),
    /// VM execution error: {error}
    ExecutionError { error: String, init_gas_cost: u64 },
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

pub(crate) use exec_bail;
