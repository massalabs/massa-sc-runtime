use wasmer::wasmparser::Operator;
use loupe::{MemoryUsage, MemoryUsageTracker};
use wasmer_types::{ExportIndex, GlobalIndex, GlobalInit, GlobalType, ModuleInfo, Mutability, Type};
use wasmer::{MiddlewareReaderState, ModuleMiddleware, MiddlewareError, LocalFunctionIndex, FunctionMiddleware};

use std::fmt::{self, Debug};
use std::mem;
use std::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug, Clone, MemoryUsage)]
struct GasCalibrationGlobalIndexes {
    imports_call_map: HashMap<u32, (String, GlobalIndex)>
}

pub struct GasCalibration {
    /// The global indexes for GasCalibration points.
    global_indexes: Mutex<Option<GasCalibrationGlobalIndexes>>,
}

#[derive(Debug)]
pub struct FunctionGasCalibration {
    /// The global indexes for GasCalibration points.
    global_indexes: GasCalibrationGlobalIndexes,
}

impl GasCalibration {
    pub fn new() -> Self {
        Self {
            global_indexes: Mutex::new(None),
        }
    }
}

impl Debug for GasCalibration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GasCalibration")
            .finish()
    }
}

impl MemoryUsage for GasCalibration {
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self) + self.global_indexes.size_of_val(tracker)
            - mem::size_of_val(&self.global_indexes)
    }
}

impl ModuleMiddleware for GasCalibration {
    fn generate_function_middleware(&self, _local_function_index: LocalFunctionIndex) -> Box<dyn FunctionMiddleware> {
        Box::new(FunctionGasCalibration {
            global_indexes: self.global_indexes.lock().unwrap().clone().unwrap(),
        })
    }

    fn transform_module_info(&self, module_info: &mut ModuleInfo) {

        let mut global_indexes = self.global_indexes.lock().unwrap();
        if global_indexes.is_some() {
            panic!("GasCalibration::transform_module_info: Attempting to use a `GasCalibration` middleware from multiple modules.");
        }

        let mut indexes = GasCalibrationGlobalIndexes { imports_call_map: Default::default() };

        for ((module_name, function_name, index), _import_index) in module_info.imports.iter() {

            // -> env.abort OR massa.assembly_script_print
            let function_fullname = format!("{}.{}", module_name, function_name);

            // Append a global for this 'imports' (== abi call) and initialize it.
            let global_index = module_info
                .globals
                .push(GlobalType::new(Type::I64, Mutability::Var));
            module_info
                .global_initializers
                .push(GlobalInit::I64Const(0));
            module_info.exports.insert(
                format!("wasmer_gascalibration_{}", function_fullname),
                ExportIndex::Global(global_index),
            );

            indexes.imports_call_map.insert(
                *index,
                (function_fullname, global_index));
        }

        *global_indexes = Some(indexes)

        // println!("module info function names: {:?}", module_info.function_names);
        // println!("module info exports: {:?}", module_info.exports);
        // println!("module info imports: {:?}", module_info.imports);
        // println!("module info functions: {:?}", module_info.functions);
    }
}

impl FunctionMiddleware for FunctionGasCalibration {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {

        // println!("Operator: {:?}", operator);

        match operator {
            Operator::Call { function_index } // function call - branch source
            => {
                println!("Got call: {}", function_index);

                let index = self
                    .global_indexes
                    .imports_call_map
                    .get(&function_index)
                    .ok_or_else(||
                        MiddlewareError::new("GasCalibration",
                                             format!("Unable to get index for function index: {}", function_index)
                        )
                    )?;

                state.extend(&[
                    Operator::GlobalGet { global_index: index.1.as_u32() },
                    Operator::I64Const { value: 1_i64 },
                    Operator::I64Add,
                    Operator::GlobalSet { global_index: index.1.as_u32() },
                ]);
            },
            // TODO: explore this
            /*
            Operator::CallIndirect { .. } // function call - branch source
            => {
                println!("Got call indirect");
            },
            */
            _ => {}
        }

        state.push_operator(operator);
        Ok(())
    }
}
