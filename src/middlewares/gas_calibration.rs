use crate::middlewares::operator::{operator_field_str, OPERATOR_VARIANTS};
use regex::{Regex, RegexSet};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::sync::Mutex;
use std::time::Instant;
use wasmer::{
    wasmparser::Operator, AsStoreMut, Extern, FunctionMiddleware, Instance, LocalFunctionIndex,
    MiddlewareError, MiddlewareReaderState, ModuleMiddleware,
};
use wasmer_types::{
    ExportIndex, GlobalIndex, GlobalInit, GlobalType, ImportIndex, ModuleInfo, Mutability, Type,
};

#[derive(Debug, Clone)]
struct GasCalibrationGlobalIndexes {
    imports_call_map: HashMap<u32, (String, GlobalIndex)>,
    op_call_map: HashMap<String, GlobalIndex>,
    param_size_map: HashMap<u32, GlobalIndex>,
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
        f.debug_struct("GasCalibration").finish()
    }
}

impl ModuleMiddleware for GasCalibration {
    fn generate_function_middleware(
        &self,
        _local_function_index: LocalFunctionIndex,
    ) -> Box<dyn FunctionMiddleware> {
        Box::new(FunctionGasCalibration {
            global_indexes: self.global_indexes.lock().unwrap().clone().unwrap(),
        })
    }

    fn transform_module_info(&self, module_info: &mut ModuleInfo) -> Result<(), MiddlewareError> {
        let current = Instant::now();

        let mut global_indexes = self.global_indexes.lock().unwrap();
        if global_indexes.is_some() {
            panic!("GasCalibration::transform_module_info: Attempting to use a `GasCalibration` middleware from multiple modules.");
        }

        // println!("{:?}", global_indexes.transform_module_info_ms);

        let mut indexes = GasCalibrationGlobalIndexes {
            imports_call_map: Default::default(),
            op_call_map: Default::default(),
            // param_size_current: global_index,
            param_size_map: Default::default(),
        };

        for (import_key, import_index) in module_info.imports.iter() {
            let module_name = import_key.module.clone();
            let function_name = import_key.field.clone();
            let index = import_key.import_idx;

            // -> env.abort OR massa.assembly_script_print
            let function_fullname = format!("{}.{}", module_name, function_name);

            // Append a global for this 'imports' (== abi call) and initialize
            // it.
            let global_index = module_info
                .globals
                .push(GlobalType::new(Type::I64, Mutability::Var));
            module_info
                .global_initializers
                .push(GlobalInit::I64Const(0));
            module_info.exports.insert(
                format!("wgc_abi_{}", function_fullname),
                ExportIndex::Global(global_index),
            );

            indexes
                .imports_call_map
                .insert(index, (function_fullname.clone(), global_index));

            // Append a global per param size per 'imports' (== abi call)
            let function_sig = if let ImportIndex::Function(f) = import_index {
                module_info.functions.get(*f).unwrap_or_else(|| {
                    panic!("Cannot get function signature for {}", function_name)
                })
            } else {
                panic!("ImportIndex is not a function??");
            };
            let function_type = module_info.signatures.get(*function_sig).unwrap();
            let param_count = function_type.params().len();

            for i in 0..param_count {
                let global_index = module_info
                    .globals
                    .push(GlobalType::new(Type::I64, Mutability::Var));
                module_info
                    .global_initializers
                    .push(GlobalInit::I64Const(0));

                module_info.exports.insert(
                    format!("wgc_ps_{}:{}", function_fullname, i),
                    ExportIndex::Global(global_index),
                );

                indexes.param_size_map.insert(index, global_index);
            }
        }

        for op_name in OPERATOR_VARIANTS {
            // Append a global for this operator and initialize it.
            let global_index = module_info
                .globals
                .push(GlobalType::new(Type::I64, Mutability::Var));
            module_info
                .global_initializers
                .push(GlobalInit::I64Const(0));
            module_info.exports.insert(
                format!("wgc_op_{}", op_name),
                ExportIndex::Global(global_index),
            );

            indexes
                .op_call_map
                .insert((*op_name).to_string(), global_index);
        }

        // println!("module info function names: {:?}",
        // module_info.function_names); println!("module info exports:
        // {:?}", module_info.exports); println!("module info imports:
        // {:?}", module_info.imports); println!("module info functions:
        // {:?}", module_info.functions); println!("module info num
        // imported functions: {:?}", module_info.num_imported_functions);
        // println!("module info start function: {:?}",
        // module_info.start_function); println!("module info passive
        // elements: {:?}", module_info.passive_elements);
        // println!("module info signatures: {:?}", module_info.signatures);

        // Append a global variable for time elapsed of this function.
        // Note: transform_module_info can take quite some time and thus we need
        // this timing for accurate gas calibration
        let global_index = module_info
            .globals
            .push(GlobalType::new(Type::F64, Mutability::Var));
        module_info
            .global_initializers
            .push(GlobalInit::F64Const(current.elapsed().as_secs_f64()));
        module_info.exports.insert(
            String::from("wgc_elapsed_transform_module_info"),
            ExportIndex::Global(global_index),
        );

        // indexes.transform_module_info_ms += duration.as_millis() as f64;
        // println!("Time elapsed in {}() is: {:?}", "transform_module_info",
        // duration);

        *global_indexes = Some(indexes);

        Ok(())
    }
}

impl FunctionMiddleware for FunctionGasCalibration {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        // let current = Instant::now();

        // println!("Operator: {:?}", operator);
        state.push_operator(operator.clone());

        if let Operator::Call { function_index } = operator {
            // let f = self.global_indexes.imports_call_map.get(&
            // function_index).unwrap(); println!("Operator::Call
            // {:?}", f);

            // state.push_operator(operator);

            if let Some(index) = self.global_indexes.imports_call_map.get(&function_index) {
                // println!("Found function index: {}", function_index);
                state.extend(&[
                    // incr function call counter
                    Operator::GlobalGet {
                        global_index: index.1.as_u32(),
                    },
                    Operator::I64Const { value: 1_i64 },
                    Operator::I64Add,
                    Operator::GlobalSet {
                        global_index: index.1.as_u32(),
                    },
                ]);
            } else {
                // Note: here we are skipping call to 'local function'
                // For instance, getOpKeys() use derOpKeys() (local) +
                // get_op_keys() (abi) Uncomment the line
                // 'println!("...", module_info.functions);' to view the list of
                // all functions (import + local)
                // Note2: Signature of function (e.g. arguments types + return
                // type) can be seen with: println!("...",
                // module_info.signatures);

                // println!("Skipping unknown function index: {}",
                // function_index);
            }
        }

        let op_name = operator_field_str(&operator);
        let index = self
            .global_indexes
            .op_call_map
            .get(op_name)
            .ok_or_else(|| {
                MiddlewareError::new(
                    "GasCalibration",
                    format!("Unable to get index for op: {}", op_name),
                )
            })?;

        state.extend(&[
            Operator::GlobalGet {
                global_index: index.as_u32(),
            },
            Operator::I64Const { value: 1_i64 },
            Operator::I64Add,
            Operator::GlobalSet {
                global_index: index.as_u32(),
            },
        ]);

        // let duration = current.elapsed();
        // println!("Time elapsed in {}() is: {:?}", "feed", duration);

        Ok(())
    }
}

#[derive(Debug)]
pub struct GasCalibrationResult {
    pub counters: HashMap<String, u64>,
    pub timers: HashMap<String, f64>,
}

// #[cfg(feature = "gas_calibration")]
pub fn get_gas_calibration_result(
    instance: &Instance,
    store: &mut impl AsStoreMut,
) -> GasCalibrationResult {
    let current = Instant::now();

    let mut result = GasCalibrationResult {
        counters: Default::default(),
        timers: Default::default(),
    };
    let patterns = [
        r"wgc_abi_([\w\.]+)",
        r"wgc_op_([\w]+)",
        r"wgc_ps_([\w\.:]+)",
        r"wgc_elapsed_([\w\._]+)",
    ];
    // Must not fail
    let set = RegexSet::new(patterns).unwrap();
    // Compile each pattern independently.
    let regexes: Vec<_> = set
        .patterns()
        .iter()
        .map(|pat| Regex::new(pat).unwrap()) // never fail, already compiled in RegexSet
        .collect();

    // let mut abi_call_index = 0;
    let mut exports_iter = instance.exports.iter();

    loop {
        let export_ = exports_iter.next();
        if export_.is_none() {
            break;
        }
        // Safe to unwrap (tested against is_none())
        let (export_name, extern_) = export_.unwrap();

        let counter_value: Option<i64> = match extern_ {
            Extern::Global(g) => g.get(store).try_into().ok(),
            _ => None,
        };
        let timer_value: Option<f64> = match extern_ {
            Extern::Global(g) => g.get(store).try_into().ok(),
            _ => None,
        };

        if counter_value.is_none() && timer_value.is_none() {
            continue;
        }

        let matches = set.matches(export_name);

        match export_name {
            ex_name if matches.matched(0) => {
                let rgx = &regexes[0];
                let mut rgx_iter = rgx.captures_iter(ex_name);

                if let Some(cap) = rgx_iter.next() {
                    if let Some(abi_func_name) = cap.get(1) {
                        result.counters.insert(
                            format!("Abi:call:{}", abi_func_name.as_str()),
                            counter_value.unwrap() as u64,
                        );
                    }
                }
            }
            ex_name if matches.matched(1) => {
                let rgx = &regexes[1];
                let mut rgx_iter = rgx.captures_iter(ex_name);

                if let Some(cap) = rgx_iter.next() {
                    if let Some(op_name) = cap.get(1) {
                        result.counters.insert(
                            format!("Wasm:{}", op_name.as_str()),
                            counter_value.unwrap() as u64,
                        );
                    }
                }
            }
            ex_name if matches.matched(2) => {
                let rgx = &regexes[2];
                let mut rgx_iter = rgx.captures_iter(ex_name);

                if let Some(cap) = rgx_iter.next() {
                    if let Some(op_name) = cap.get(1) {
                        result.counters.insert(
                            format!("Abi:ps:{}", op_name.as_str()),
                            counter_value.unwrap() as u64,
                        );
                    }
                }
            }
            ex_name if matches.matched(3) => {
                let rgx = &regexes[3];
                let mut rgx_iter = rgx.captures_iter(ex_name);

                if let Some(cap) = rgx_iter.next() {
                    if let Some(fn_name) = cap.get(1) {
                        result
                            .timers
                            .insert(format!("Time:{}", fn_name.as_str()), timer_value.unwrap());
                    }
                }
            }
            _ => {
                println!("Unhandled export name: {}", export_name);
            }
        }
    }

    let duration = current.elapsed();
    result.timers.insert(
        String::from("Time:gas_calibration_result"),
        duration.as_secs_f64(),
    );

    // println!("Time elapsed in {}() is: {:?}", "gas_calibration_result",
    // duration);

    result
}

// pub(crate) fn _param_size_update(
// env: &ASEnv,
// store: &mut impl AsStoreMut,
// function_name: &str,
// param_len: usize,
// is_str: bool,
// ) {
// println!("Calling param_size_update: {} {}", function_name, param_len);
// let function_full_name = format!("wgc_ps_{}", function_name);
// let global_ref = env
// .get_gc_param(&function_full_name)
// .unwrap_or_else(|| panic!("Unable to find counter for {}",
// function_full_name)); Cannot fail as the middleware define it as i64
// let v: i64 = global_ref
// .get(store)
// .try_into()
// .expect("Unable to convert global ref to i64");
//
// AssemblyScript use utf-16 string so we need to mult by 2
// let s = v + match is_str {
// true => (param_len * 2) as i64,
// false => param_len as i64,
// };
// let error_msg = format!("Unable to set global {:?} with value: {}",
// global_ref, s); global_ref.set(store, s.into()).expect(&error_msg);
// }
