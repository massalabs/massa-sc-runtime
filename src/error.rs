use displaydoc::Display;
use thiserror::Error;

pub type VMResult<T> = Result<T, VMError>;

#[derive(Clone, Error, Display, Debug)]
pub enum VMError {
    /// VM instance error: {0}
    InstanceError(String),
    /// VM execution error: {error} | Init cost is {init_cost}
    ExecutionError { error: String, init_cost: u64 },
}

impl From<anyhow::Error> for VMError {
    fn from(value: anyhow::Error) -> Self {
        Self::InstanceError(value.to_string())
    }
}

macro_rules! exec_bail {
    ($err:expr, $init_cost:expr) => {
        return Err(crate::VMError::ExecutionError {
            error: $err.to_string(),
            init_cost: $init_cost,
        })
    };
}

pub(crate) use exec_bail;
