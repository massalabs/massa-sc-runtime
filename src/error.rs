use displaydoc::Display;
use thiserror::Error;

pub type RuntimeResult<T, E = RuntimeError> = core::result::Result<T, E>;

#[derive(Display, Error, Debug)]
pub enum RuntimeError {
    /// Runtime error: {0}
    Error(#[from] anyhow::Error),
    /// Wasmer runtime error: {0}
    RuntimeError(#[from] wasmer::RuntimeError),
}
