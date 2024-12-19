/// An entry-point protection middleware that prevents malicious WASM files from reaching the compilation step.
use std::fmt;
use wasmer::{
    FunctionMiddleware, LocalFunctionIndex, MiddlewareError, MiddlewareReaderState,
    ModuleMiddleware,
};
use wasmer_types::ModuleInfo;

use crate::CondomLimits;

/// The module-level export limit middleware, named `CondomMiddleware`.
pub struct CondomMiddleware {
    /// Maximum allowed number of exports.
    limits: CondomLimits,
}

impl CondomMiddleware {
    /// Creates a new `CondomMiddleware`.
    pub fn new(limits: CondomLimits) -> Self {
        Self { limits }
    }
}

impl fmt::Debug for CondomMiddleware {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CondomMiddleware")
            .field("limits", &self.limits)
            .finish()
    }
}

impl ModuleMiddleware for CondomMiddleware {
    /// This method is called before applying middleware on functions,
    /// and it will enforce the export limits on the module.
    fn transform_module_info(&self, module_info: &mut ModuleInfo) -> Result<(), MiddlewareError> {
        // Check the export limits and return an error if exceeded
        if let Some(max_exports) = self.limits.max_exports {
            if module_info.exports.len() > max_exports {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    "The WASM file has too many exports. Blocked to prevent compile-time memory bloat",
                ));
            }
        }

        // Check the number of function limits
        if let Some(max_functions) = self.limits.max_functions {
            if module_info.functions.len() > max_functions {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    "The WASM file has too many functions. Blocked to prevent compile-time memory bloat",
                ));
            }
        }

        // Check the signature limits i.e. the number of arguments a function
        // can take.
        // NOTE, at the moment (2024-09-30) :
        // - functions can returns only one value, see:
        // https://developer.mozilla.org/en-US/docs/WebAssembly/Understanding_the_text_format#signatures_and_parameters
        // - tweaking test_condom_middleware_exceeds_args_limit_max(), one can
        //   see that wasmer limits number of arguments to 1024
        if let Some(max_signature_len) = self.limits.max_signature_len {
            for signature in module_info.signatures.values() {
                if signature.params().len() + signature.results().len() > max_signature_len {
                    return Err(MiddlewareError::new(
                        "CondomMiddleware",
                        "The WASM file contains a function with too many parameters and return values. Blocked to prevent compile-time memory bloat",
                    ));
                }
            }
        }

        // Check the name limit
        if let Some(max_name_len) = self.limits.max_name_len {
            if let Some(name) = &module_info.name {
                if name.len() > max_name_len {
                    dbg!(max_name_len, name.len());
                    return Err(MiddlewareError::new(
                        "CondomMiddleware",
                        "The WASM file has a too long name. Blocked to prevent compile-time memory bloat",
                    ));
                }
            }
        }

        // Check the import limits
        if let Some(max_imports) = self.limits.max_imports_len {
            if module_info.imports.len() > max_imports {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    "The WASM file has too many imports. Blocked to prevent compile-time memory bloat",
                ));
            }
        }

        // Check the table initializer limits
        if let Some(max_table_initializers) = self.limits.max_table_initializers_len {
            if module_info.table_initializers.len() > max_table_initializers {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    "The WASM file has too many table initializers. Blocked to prevent compile-time memory bloat",
                ));
            }
        }

        // Check passive elements limits
        if let Some(max_passive_elements) = self.limits.max_passive_elements_len {
            if module_info.passive_elements.len() > max_passive_elements {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    "The WASM file has too many passive elements. Blocked to prevent compile-time memory bloat",
                ));
            }
        }

        // Check passive data limits
        if let Some(max_passive_data) = self.limits.max_passive_data_len {
            if module_info.passive_data.len() > max_passive_data {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    "The WASM file has too many passive data. Blocked to prevent compile-time memory bloat",
                ));
            }
        }

        // Check the global initializer limits
        if let Some(max_global_initializers) = self.limits.max_global_initializers_len {
            if module_info.global_initializers.len() > max_global_initializers {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    format!("The WASM file has too many global initializers (contains; {} limit: {}). Blocked to prevent compile-time memory bloat",
                        module_info.global_initializers.len(),
                        max_global_initializers,
                    ),
                ));
            }
        }

        // Check the function name limits
        if let Some(max_function_names) = self.limits.max_function_names_len {
            for (_, f_name) in module_info.function_names.iter() {
                if f_name.len() > max_function_names {
                    return Err(MiddlewareError::new(
                        "CondomMiddleware",
                        format!("The WASM file has too long function names ({} is {}). Blocked to prevent compile-time memory bloat",
                            f_name,
                            f_name.len()),
                ));
                }
            }
        }

        // Check the table limits
        if let Some(max_tables) = self.limits.max_tables_count {
            if module_info.tables.len() > max_tables {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    format!("The WASM file has too many tables (contains: {} limit: {}). Blocked to prevent compile-time memory bloat",
                    module_info.tables.len(),
                    max_tables,
                    ),
                ));
            }
        }

        // Check the memory limits
        if let Some(max_memories) = self.limits.max_memories_len {
            if module_info.memories.len() > max_memories {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    "The WASM file has too many memories. Blocked to prevent compile-time memory bloat",
                ));
            }
        }

        // Check the global limits
        if let Some(max_globals) = self.limits.max_globals_len {
            if module_info.globals.len() > max_globals {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    format!("The WASM file has too many globals (contains: {}, limit: {}). Blocked to prevent compile-time memory bloat",
                        module_info.globals.len(),
                        max_globals
                    )
                ));
            }
        }

        // custom_sections: IndexMap<String, CustomSectionIndex>,
        // Check the custom section limits
        if let Some(max_custom_sections) = self.limits.max_custom_sections_len {
            if module_info.custom_sections.len() > max_custom_sections {
                return Err(MiddlewareError::new(
                    "CondomMiddleware",
                    "The WASM file has too many custom sections. Blocked to prevent compile-time memory bloat",
                ));
            }
        }

        // Check the custom section data limits
        if let Some(max_custom_sections_data) = self.limits.max_custom_sections_data_len {
            for (idx, data) in module_info.custom_sections_data.iter() {
                if data.len() > max_custom_sections_data {
                    let section_name = module_info
                        .custom_sections
                        .get_index(idx.as_u32() as usize)
                        .map(|(name, _)| name.to_owned())
                        .unwrap_or("Unknown section".to_string());

                    return Err(MiddlewareError::new(
                        "CondomMiddleware",
                        format!("The WASM file custom section {:?} named '{}' with size {} is too big (limit: {}). Blocked to prevent compile-time memory bloat",
                            idx,
                            section_name,
                            data.len(),
                            max_custom_sections_data,
                        ),
                    ));
                };
            }
        }

        Ok(())
    }

    /// Generates a `FunctionMiddleware` for a given function.
    fn generate_function_middleware(&self, _: LocalFunctionIndex) -> Box<dyn FunctionMiddleware> {
        // This middleware doesn't need to modify functions, so we return a no-op middleware.
        Box::new(NoOpFunctionMiddleware)
    }
}

/// A no-op function middleware used as a placeholder.
struct NoOpFunctionMiddleware;

impl fmt::Debug for NoOpFunctionMiddleware {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoOpFunctionMiddleware").finish()
    }
}

impl FunctionMiddleware for NoOpFunctionMiddleware {
    fn feed<'a>(
        &mut self,
        operator: wasmer::wasmparser::Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        state.push_operator(operator);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use wasmer::{sys::EngineBuilder, wat2wasm, CompilerConfig, Cranelift, Module, Store};

    const MEMORY_LIMIT_1GB: u64 = 1024 * 1024 * 1024;

    fn bytecode_within_limits() -> Vec<u8> {
        wat2wasm(
            br#"
            (module
                (func (export "f1") (result i32) (i32.const 1))
                (func (export "f2") (result i32) (i32.const 2))
                (global (export "g1") i32 (i32.const 0))
            )
            "#,
        )
        .unwrap()
        .into()
    }

    fn bytecode_exceeding_exports_limits() -> Vec<u8> {
        wat2wasm(
            br#"
            (module
                (func (export "f1") (result i32) (i32.const 1))
                (func (export "f2") (result i32) (i32.const 2))
                (func (export "f3") (result i32) (i32.const 3))
                (global (export "g1") i32 (i32.const 0))
                (global (export "g2") i32 (i32.const 0))
            )
            "#,
        )
        .unwrap()
        .into()
    }

    fn generate_wasm_bloated_with_functions(nb_fn: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");
        for i in 1..=nb_fn {
            wasm_code.push_str(&format!("  (func (result i32) (i32.const {}))\n", i));
        }
        wasm_code.push(')');
        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_bloated_with_args(nb_args: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        if nb_args == 0 {
            wasm_code.push_str("  (func $f (result i32) (i32.const 0))\n");
        } else {
            wasm_code.push_str("  (func $f (param");
            for _ in 1..nb_args {
                wasm_code.push_str(" i32 ");
            }
            wasm_code.push_str(") (result i32)\n");
            wasm_code.push_str("    local.get 0\n");
        }

        wasm_code.push_str("  )\n)");

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_bloated_with_table_initializers(table_size: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        wasm_code.push_str("  (func $f (result i32) (i32.const 42))\n");
        wasm_code.push_str(&format!("  (table {} funcref)\n", table_size));

        for i in 0..table_size {
            wasm_code.push_str(&format!("  (elem (i32.const {}) $f)\n", i));
        }
        wasm_code.push(')');

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_bloated_with_variable_passive_elements(passive_count: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        wasm_code.push_str("  (func $f (result i32) (i32.const 42))\n");

        for _ in 0..passive_count {
            wasm_code.push_str("  (elem func $f)\n");
        }

        wasm_code.push(')');

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_with_variable_passive_data(data_size: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        wasm_code.push_str("  (memory $m 1)\n");

        for i in 0..data_size {
            wasm_code.push_str(&format!("(data $d{} \"Hello, World!\")\n", i));
        }

        wasm_code.push_str(")\n");

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_with_global_initializers(data_size: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        for i in 0..data_size {
            wasm_code.push_str(&format!(
                "(global $g{} (export \"g{}\") i32 (i32.const 42))\n",
                i, i
            ));
        }

        wasm_code.push_str(")\n");

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_with_function_names(data_size: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        let fn_name = "f".repeat(data_size);

        wasm_code.push_str(&format!(
            "(func ${} (result i32) (i32.const 42))\n",
            fn_name
        ));

        wasm_code.push_str(")\n");

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_bloated_with_tables(table_count: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        for _ in 0..table_count {
            wasm_code.push_str("  (table 1 funcref)\n");
        }
        wasm_code.push(')');

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_bloated_with_memories(memories_count: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        for i in 0..memories_count {
            wasm_code.push_str(&format!(
                "  (import \"m{}\" \"memory{}\" (memory 1))\n",
                i, i
            ));
        }
        wasm_code.push(')');

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_bloated_with_globals(globals_count: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        for i in 0..globals_count {
            wasm_code.push_str(&format!("(global $g{} (mut i32) (i32.const 10))", i));
        }
        wasm_code.push(')');

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_with_multiple_returns(nb_returns: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        if nb_returns == 0 {
            wasm_code.push_str("  (func $f (result))\n");
        } else {
            wasm_code.push_str("  (func $f (result");
            for _ in 0..nb_returns {
                wasm_code.push_str(" i32");
            }
            wasm_code.push_str(")\n");

            for i in 0..nb_returns {
                wasm_code.push_str(&format!("    i32.const {}\n", i));
            }

            wasm_code.push_str("  )\n");
        }

        wasm_code.push(')');

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    fn generate_wasm_with_local_vars(nb_locals: usize) -> Vec<u8> {
        let mut wasm_code = String::from("(module\n");

        wasm_code.push_str("  (func $f (result i32)\n");

        if nb_locals > 0 {
            wasm_code.push_str("    (local");
            for _ in 0..nb_locals {
                wasm_code.push_str(" i32");
            }
            wasm_code.push_str(")\n");
        }

        for i in 0..nb_locals {
            wasm_code.push_str(&format!("    i32.const 0\n    local.set {}\n", i));
        }

        if nb_locals > 0 {
            wasm_code.push_str("    local.get 0\n");
        } else {
            wasm_code.push_str("    i32.const 0\n");
        }

        wasm_code.push_str("  )\n");
        wasm_code.push(')');

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    #[test]
    fn test_condom_middleware_exceeds_exports_limit() {
        let condom_limits = CondomLimits {
            max_exports: Some(2),
            ..Default::default()
        };
        let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits));
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(condom_middleware);

        let store = Store::new(EngineBuilder::new(compiler_config));
        let module_result = Module::new(&store, bytecode_exceeding_exports_limits());

        assert!(module_result.is_err());
        if let Err(e) = module_result {
            assert!(e.to_string().contains("too many exports"));
        }
    }

    use sysinfo::{Pid, System};
    fn get_memory_usage() -> Result<u64, String> {
        let mut system = System::new_all();
        system.refresh_all();
        let pid = Pid::from_u32(std::process::id());
        if let Some(process) = system.process(pid) {
            Ok(process.memory())
        } else {
            Err("Proccess not found".to_string())
        }
    }

    use num_format::{SystemLocale, ToFormattedString};
    #[test]
    fn test_condom_middleware_exceeds_functions_limit_max() {
        let condom_limits = CondomLimits {
            max_functions: Some(10000),
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        // starting from 1000 functions
        let mut function_count = 1000;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result =
                Module::new(&store, generate_wasm_bloated_with_functions(function_count));

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "functions {:>8} memory usage: {:>12}o ",
                function_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many functions"));
                }
                break;
            }

            function_count *= 10;
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_args_limit_max() {
        let condom_limits = CondomLimits {
            max_signature_len: Some(500),
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        // starting from 2 arguments
        let mut arg_count = 2;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result = Module::new(&store, generate_wasm_bloated_with_args(arg_count));

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "args {:>4} memory usage: {:>12}o ",
                arg_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many parameters"));
                }
                break;
            }

            arg_count *= 2;
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_return_values_limit_max() {
        let condom_limits = CondomLimits {
            max_signature_len: Some(500),
            ..Default::default()
        };

        let mut arg_count = 2;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result = Module::new(&store, generate_wasm_with_multiple_returns(arg_count));

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "args {:>4} memory usage: {:>12}o ",
                arg_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many parameters"));
                }
                break;
            }

            arg_count *= 2;
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_table_initializers_limit_max() {
        let condom_limits = CondomLimits {
            max_table_initializers_len: Some(100),
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        let mut ini_count = 10;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result = Module::new(
                &store,
                generate_wasm_bloated_with_table_initializers(ini_count),
            );

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "Initializers {:>6} memory usage: {:>12}o ",
                ini_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many table initializers"));
                }
                break;
            }

            ini_count *= 10;
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_passive_elements_limit_max() {
        let condom_limits = CondomLimits {
            max_passive_elements_len: Some(100),
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        let mut ini_count = 10;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result = Module::new(
                &store,
                generate_wasm_bloated_with_variable_passive_elements(ini_count),
            );

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "Passive elements {:>6} memory usage: {:>12}o ",
                ini_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many passive elements"));
                }
                break;
            }

            ini_count *= 10;
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_passive_data_limit_max() {
        let condom_limits = CondomLimits {
            max_passive_data_len: Some(10000),
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        let mut ini_count = 10;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result =
                Module::new(&store, generate_wasm_with_variable_passive_data(ini_count));

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "Passive data {:>6} memory usage: {:>12}o ",
                ini_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many passive data"));
                }
                break;
            }

            ini_count *= 10;
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_global_initializers_limit_max() {
        let condom_limits = CondomLimits {
            max_global_initializers_len: Some(10000),
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        let mut ini_count = 10;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result =
                Module::new(&store, generate_wasm_with_global_initializers(ini_count));

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "Global initializers {:>6} memory usage: {:>12}o ",
                ini_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many global initializers"));
                }
                break;
            }

            ini_count *= 10;
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_function_names_limit_max() {
        let condom_limits = CondomLimits {
            max_function_names_len: Some(99999),
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        let mut fn_name_count = 10;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result =
                Module::new(&store, generate_wasm_with_function_names(fn_name_count));

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "Function names {:>8} memory usage: {:>12}o ",
                fn_name_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too long function names"));
                }
                break;
            }

            fn_name_count *= 10;
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_tables_limit_max() {
        let condom_limits = CondomLimits {
            max_tables_count: Some(15),
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        let mut table_count = 2;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result = Module::new(&store, generate_wasm_bloated_with_tables(table_count));

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "Table count {:>6} memory usage: {:>12}o ",
                table_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many tables"));
                }
                break;
            }

            table_count *= 2;
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_memories_limit_max() {
        let condom_limits = CondomLimits {
            // NOTE: max_memories_len is defined to Some(0) because the
            // specification, FOR NOW, only support ONE memory. That being said
            // wasmer seems ready for extension.
            max_memories_len: Some(0),
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        let mut memory_count = 1;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result =
                Module::new(&store, generate_wasm_bloated_with_memories(memory_count));

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "Memory count {:>6} memory usage: {:>12}o ",
                memory_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many memories"));
                }
                break;
            }

            memory_count *= 2;
        }
    }
    #[test]
    fn test_condom_middleware_exceeds_globals_limit_max() {
        let condom_limits = CondomLimits {
            max_globals_len: Some(1000),
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        let mut memory_count = 10;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result =
                Module::new(&store, generate_wasm_bloated_with_globals(memory_count));

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "Global count {:>6} memory usage: {:>12}o ",
                memory_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many globals"));
                }
                break;
            }

            memory_count *= 10;
        }
    }
    #[test]
    fn test_condom_middleware_exceeds_function_locales_limit_max() {
        // NOTE: can't find a way to set limit for locals. We are currently
        // relying on the wasmpaser limit that is set to 50_000 locales
        // (https://github.com/bytecodealliance/wasm-tools/blob/610b7aacdb41f129b8dda3f04785934105bd6791/crates/wasmprinter/src/lib.rs#L20)
        // This should do the trick as
        // 50     locals use 51 134 464b of ram
        // 500000 locals use 90 607 616b of ram
        let condom_limits = CondomLimits {
            ..Default::default()
        };

        // Keep increasing the wasm size until the memory usage is above the limit
        let mut memory_count = 50;
        loop {
            let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
            let mut compiler_config = Cranelift::default();
            compiler_config.push_middleware(condom_middleware);

            let store = Store::new(EngineBuilder::new(compiler_config));
            let module_result = Module::new(&store, generate_wasm_with_local_vars(memory_count));

            let memory_usage = get_memory_usage().unwrap();
            println!(
                "Locals count {:>6} memory usage: {:>12}o ",
                memory_count,
                memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
            );

            if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
                assert!(module_result.is_err());
                // dbg!(&module_result);
                if let Err(e) = module_result {
                    assert!(e.to_string().contains("too many locals"));
                }
                break;
            }

            memory_count *= 10;
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_custom_sections_limit_max() {
        // NOTE:
        // simple-custom-section.wasm can be generated in massa-unit-test-src repository

        let condom_limits = CondomLimits {
            max_custom_sections_len: Some(2),
            ..Default::default()
        };

        let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(condom_middleware);

        let store = Store::new(EngineBuilder::new(compiler_config));
        let wasm = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/wasm/simple-custom-section.wasm"
        ));
        let module_result = Module::new(&store, wasm);

        let memory_usage = get_memory_usage().unwrap();
        println!(
            "Memory usage: {:>12}o ",
            memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
        );

        if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
            assert!(module_result.is_err());
            if let Err(e) = module_result {
                assert!(e.to_string().contains("too many custom sections"));
            }
        }
    }

    #[test]
    fn test_condom_middleware_exceeds_custom_sections_data_limit_max() {
        // NOTE:
        // simple-custom-section.wasm can be generated in massa-unit-test-src repository

        let condom_limits = CondomLimits {
            max_custom_sections_data_len: Some(9),
            ..Default::default()
        };

        let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(condom_middleware);

        let store = Store::new(EngineBuilder::new(compiler_config));
        let wasm = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/wasm/simple-custom-section.wasm"
        ));
        let module_result = Module::new(&store, wasm);

        let memory_usage = get_memory_usage().unwrap();
        println!(
            "Memory usage: {:>12}o ",
            memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
        );

        if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
            assert!(module_result.is_err());
            if let Err(e) = module_result {
                assert!(e.to_string().contains("is too big"));
            }
        }
    }

    fn generate_wasm_calibrate(
        nb_fn: usize,
        fn_name_len: usize,
        nb_params: usize,
        nb_return_values: usize,
        nb_locals: usize,
        nb_exports_fn: usize,
        nb_imports_fn: usize,
        nb_table_ini: usize,
        nb_passive_elm: usize,
        nb_passive_data: usize,
        nb_global_ini: usize,
        nb_tables: usize,
    ) -> Vec<u8> {
        assert!(fn_name_len > 0);

        fn gen_fn_names(count: usize, length: usize) -> Vec<String> {
            use rand::distributions::Alphanumeric;
            use rand::thread_rng;
            use rand::Rng;
            use std::collections::HashSet;

            let mut unique_strings = HashSet::new();
            let mut rng = thread_rng();

            while unique_strings.len() < count {
                // Générer une chaîne de caractères aléatoire
                let random_string: String = (0..length)
                    .map(|_| rng.sample(Alphanumeric) as char)
                    .collect();

                // Ajouter la chaîne générée à l'ensemble
                unique_strings.insert(random_string);
            }

            unique_strings.into_iter().collect::<Vec<_>>()
        }
        let fn_names = gen_fn_names(nb_fn, fn_name_len);

        let mut wasm_code = String::from("(module\n");

        // import mem
        wasm_code.push_str(";; import memory\n");
        wasm_code.push_str("  (import \"env\" \"memory\" (memory 1 10))\n");
        wasm_code.push('\n');

        // import fn
        wasm_code.push_str(";; import functions\n");
        for i in 0..nb_imports_fn {
            wasm_code.push_str(&format!(
                "  (import \"env\" \"external_fn{}\" (func (param i32 i32) (result i32)))\n",
                i
            ));
        }
        wasm_code.push('\n');

        // passive data
        wasm_code.push_str(";; passive data\n");
        for i in 0..nb_passive_data {
            wasm_code.push_str(&format!("(data $d{} \"Hello, World!\")\n", i));
        }
        wasm_code.push('\n');

        // global initializers
        wasm_code.push_str(";; global initializers\n");
        for i in 0..nb_global_ini {
            wasm_code.push_str(&format!("(global $g{} i32 (i32.const 42))\n", i,));
        }
        wasm_code.push('\n');

        // table
        wasm_code.push_str(";; table\n");
        for _ in 0..nb_tables {
            wasm_code.push_str("  (table 1 funcref)\n");
        }
        wasm_code.push('\n');

        // decl fn
        wasm_code.push_str(";; declare functions\n");
        for i in 0..nb_fn {
            wasm_code.push_str(&format!("  (func ${} (param", fn_names.get(i).unwrap()));

            for _ in 0..nb_params {
                wasm_code.push_str(" i32");
            }
            wasm_code.push_str(") (result");

            for _ in 0..nb_return_values {
                wasm_code.push_str(" i32");
            }
            wasm_code.push_str(")\n");

            for _ in 0..nb_locals {
                wasm_code.push_str("    (local i32)\n");
            }

            wasm_code.push_str("    (block (result");
            for _ in 0..nb_return_values {
                wasm_code.push_str(" i32");
            }
            wasm_code.push_str(")\n");

            for _ in 0..nb_return_values {
                wasm_code.push_str(&format!("      i32.const {}\n", i));
            }

            wasm_code.push_str("    )\n  )\n");
        }
        wasm_code.push('\n');

        // export fn
        wasm_code.push_str(";; export functions\n");
        for i in 0..nb_exports_fn {
            wasm_code.push_str(&format!(
                "  (export \"fn_{}\" (func ${}))\n",
                i,
                fn_names.get(i).unwrap()
            ));
        }
        wasm_code.push('\n');

        // table initializer
        wasm_code.push_str(";; table initializers\n");
        wasm_code.push_str(&format!("  (table {} funcref)\n", nb_table_ini));
        for i in 0..nb_table_ini {
            wasm_code.push_str(&format!(
                "    (elem (i32.const {}) ${})\n",
                i,
                fn_names.get(i).unwrap()
            ));
        }
        wasm_code.push('\n');

        // passive elements
        wasm_code.push_str(";; passive elements\n");
        for i in 0..nb_passive_elm {
            wasm_code.push_str(&format!("  (elem func ${})\n", fn_names.get(i).unwrap()));
        }

        wasm_code.push(')');

        std::fs::write("calibrate.wat", &wasm_code).unwrap();

        wat2wasm(wasm_code.as_bytes()).unwrap().into()
    }

    #[test]
    fn test_condom_middleware_calibrate() {
        use std::time::Instant;
        let nb_fn = 512;
        let nb_params = 64;
        let nb_return_values = 8;
        let name_len = 256;
        let fn_name_len = 256;
        let custon_section_data_len = 1_000_000;
        let nb_exports = nb_fn;
        let nb_imports_fn = 256;
        let nb_imports_mem = 1;
        let nb_table_ini = nb_fn;
        let nb_passive_elm = nb_fn;
        let nb_passive_data = 512;
        let nb_global_ini = 512;
        let nb_tables = 16;
        let nb_memories = 1; // only 1 supported so far (cf specification)

        let nb_locals = 512;

        let condom_limits = CondomLimits {
            max_exports: Some(nb_exports + nb_global_ini),
            max_functions: Some(nb_fn + nb_imports_fn),
            max_signature_len: Some(nb_params + nb_return_values),
            max_name_len: Some(name_len),
            max_imports_len: Some(nb_imports_fn + nb_imports_mem),
            max_table_initializers_len: Some(nb_table_ini),
            max_passive_elements_len: Some(nb_passive_elm),
            max_passive_data_len: Some(nb_passive_data),
            max_global_initializers_len: Some(nb_global_ini),
            max_function_names_len: Some(fn_name_len),
            max_tables_count: Some(nb_tables + 1), // +1 for the table with initializers
            max_memories_len: Some(nb_memories),
            max_globals_len: Some(nb_global_ini),
            max_custom_sections_len: Some(1),
            max_custom_sections_data_len: Some(custon_section_data_len),
        };

        let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits.clone()));
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(condom_middleware);

        let store = Store::new(EngineBuilder::new(compiler_config));
        let wasm_bytes = generate_wasm_calibrate(
            nb_fn,
            fn_name_len,
            nb_params,
            nb_return_values,
            nb_locals,
            nb_exports,
            nb_imports_fn,
            nb_table_ini,
            nb_passive_elm,
            nb_passive_data,
            nb_global_ini,
            nb_tables,
        );

        let start_time = Instant::now();
        let module_result = Module::new(&store, wasm_bytes);
        let compilation_duration = Instant::now().duration_since(start_time);
        println!("Module creation took: {:?}", compilation_duration);

        let memory_usage = get_memory_usage().unwrap();
        println!(
            "Memory usage: {:>12}o ",
            memory_usage.to_formatted_string(&SystemLocale::default().unwrap()),
        );

        if memory_usage > MEMORY_LIMIT_1GB || module_result.is_err() {
            dbg!(&module_result);
            assert!(module_result.is_err());
            if let Err(e) = module_result {
                assert!(e.to_string().contains("is too big"));
            }
        }
    }

    #[test]
    fn test_condom_middleware_exports_within_limits() {
        let condom_limits = CondomLimits {
            max_exports: Some(3),
            ..Default::default()
        };
        let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits));
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(condom_middleware);

        let store = Store::new(EngineBuilder::new(compiler_config));
        let module_result = Module::new(&store, bytecode_within_limits());

        assert!(module_result.is_ok());
    }
}
