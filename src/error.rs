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
        // Only a depth error must keep its variant: any other ABI error trapping out of a
        // host function is a regular failure and must not be reported as a depth error.
        // The other arms mirror `From<ABIError> for VMError` so that an already formatted
        // message is not prefixed twice.
        match e.downcast_ref::<ABIError>() {
            Some(ABIError::DepthError(err)) => VMError::DepthError(err.clone()),
            Some(
                ABIError::VMError(err) | ABIError::RuntimeError(err) | ABIError::SerdeError(err),
            ) => VMError::InstanceError(err.clone()),
            Some(ABIError::Error(err)) => VMError::InstanceError(err.to_string()),
            None => VMError::InstanceError(e.to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A depth error trapping out of a host function must keep its variant and its message.
    #[test]
    fn test_depth_error_is_preserved() {
        let err = wasmer::RuntimeError::user(Box::new(ABIError::DepthError(
            "recursion depth limit reached".to_string(),
        )));
        match VMError::from(err) {
            VMError::DepthError(msg) => assert_eq!(msg, "recursion depth limit reached"),
            e => panic!("expected a depth error, got: {e}"),
        }
    }

    /// Any other ABI error must not be reported as a depth error, and must not be prefixed twice.
    #[test]
    fn test_other_abi_errors_are_not_depth_errors() {
        let err = wasmer::RuntimeError::user(Box::new(ABIError::VMError(
            "VM instance error: RuntimeError: unreachable".to_string(),
        )));
        match VMError::from(err) {
            VMError::InstanceError(msg) => {
                assert_eq!(msg, "VM instance error: RuntimeError: unreachable")
            }
            e => panic!("expected an instance error, got: {e}"),
        }
    }

    /// A trap that carries no ABI error at all is an instance error.
    #[test]
    fn test_plain_trap_is_an_instance_error() {
        let err = wasmer::RuntimeError::new("unreachable");
        assert!(matches!(VMError::from(err), VMError::InstanceError(_)));
    }
}
