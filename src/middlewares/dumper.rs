use std::fmt::{self, Debug};
use wasmer::{
    wasmparser::Operator, FunctionMiddleware, LocalFunctionIndex, MiddlewareError,
    MiddlewareReaderState, ModuleMiddleware,
};
use wasmer_types::ModuleInfo;

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

    fn transform_module_info(&self, _module_info: &mut ModuleInfo) -> Result<(), MiddlewareError> {
        println!("Transform module info...");

        Ok(())
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
