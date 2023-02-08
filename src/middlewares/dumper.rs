use wasmer::wasmparser::Operator;
use wasmer::{
    AsStoreMut, FunctionMiddleware, LocalFunctionIndex, MiddlewareError, MiddlewareReaderState,
    ModuleMiddleware,
};
use wasmer_types::{
    ExportIndex, GlobalIndex, GlobalInit, GlobalType, ImportIndex, ModuleInfo, Mutability, Type,
};

use crate::env::{ASEnv, MassaEnv};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::sync::Mutex;
use std::time::Instant;

use crate::middlewares::operator::{operator_field_str, OPERATOR_VARIANTS};

// #[cfg(feature = "gas_calibration")]
use regex::{Regex, RegexSet};
// #[cfg(feature = "gas_calibration")]
use wasmer::{Extern, Instance};

pub struct Dumper {}

#[derive(Debug)]
pub struct FunctionDumper {}

impl Dumper {
    pub fn new() -> Self {
        Self {}
    }
}

impl Debug for Dumper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dumper").finish()
    }
}

impl ModuleMiddleware for Dumper {
    fn generate_function_middleware(
        &self,
        _local_function_index: LocalFunctionIndex,
    ) -> Box<dyn FunctionMiddleware> {
        Box::new(FunctionDumper {})
    }

    fn transform_module_info(&self, module_info: &mut ModuleInfo) {
        println!("Transform module info...");
    }
}

impl FunctionMiddleware for FunctionDumper {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        println!("{:?}", operator);
        state.push_operator(operator.clone());
        Ok(())
    }
}
