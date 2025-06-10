use crate::as_execution::{exec_as_module, ASModule};
use crate::error::VMResult;
use crate::middlewares::gas_calibration::GasCalibrationResult;
use crate::types::{Interface, Response};
use crate::wasmv1_execution::{exec_wasmv1_module, WasmV1Module};
use crate::{settings, CondomLimits};
use crate::{GasCosts, VMError};
use anyhow::{anyhow, Result};
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Enum listing the available compilers
#[derive(Clone)]
pub enum Compiler {
    CL,
    SP,
}

#[derive(Clone)]
pub enum RuntimeModule {
    ASModule(ASModule),
    WasmV1Module(WasmV1Module),
}

#[repr(u8)]
#[derive(IntoPrimitive, Debug, Eq, PartialEq, TryFromPrimitive)]

enum RuntimeModuleId {
    ASModuleId = 0,
    WasmV1ModuleId = 1,
}

impl RuntimeModule {
    /// Dispatch module creation corresponding to the first bytecode byte
    ///
    /// * (0): legacy AssemblyScript module
    /// * (1): new agnostic module
    /// * (_): unsupported module
    pub fn new(
        bytecode: &[u8],
        gas_costs: GasCosts,
        compiler: Compiler,
        condom_limits: CondomLimits,
    ) -> Result<Self> {
        if bytecode.len() <= 2 {
            return Err(anyhow!("Too small bytecode"));
        }

        let module_id = bytecode
            .first()
            .map(|&id| RuntimeModuleId::try_from(id))
            .transpose()
            .map_err(|err| anyhow!("Unsupported file format for SC({})", err))?
            .unwrap(); // Safe to unwrap as we checked the bytecode length and for conversion
                       // errors

        match module_id {
            RuntimeModuleId::ASModuleId => Ok(Self::ASModule(ASModule::new(
                bytecode,
                gas_costs.max_instance_cost,
                gas_costs,
                compiler,
                condom_limits,
            )?)),
            RuntimeModuleId::WasmV1ModuleId => {
                // Safe to use [1..] as we checked the bytecode length
                // TODO: ensure that the WasmV1 VM can be refilled with gas after launching a pre-compiled module
                let res = WasmV1Module::compile(
                    &bytecode[1..],
                    gas_costs.max_instance_cost,
                    gas_costs,
                    compiler,
                    condom_limits,
                )
                .map_err(|err| anyhow!("Failed to compile WasmV1 module: {}", err))?;
                Ok(Self::WasmV1Module(res))
            }
        }
    }

    /// Used compiler for the current module
    pub fn compiler(&self) -> Compiler {
        match self {
            RuntimeModule::ASModule(module) => module.compiler.clone(),
            RuntimeModule::WasmV1Module(module) => module.compiler.clone(),
        }
    }

    /// Serialize a RuntimeModule, prepending its byte id
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let (mut ser, id) = match self {
            RuntimeModule::ASModule(module) => (module.serialize()?, RuntimeModuleId::ASModuleId),
            RuntimeModule::WasmV1Module(module) => {
                (module.serialize(), RuntimeModuleId::WasmV1ModuleId)
            }
        };

        // CHECK: as we know the size before hand, maybe we can put the id at
        // the end of the vector, this would prevent a memmove
        ser.insert(0, id as u8);

        Ok(ser)
    }

    /// Deserialize a RuntimeModule
    pub fn deserialize(
        ser_module: &[u8],
        limit: u64,
        gas_costs: GasCosts,
        condom_limits: CondomLimits,
    ) -> Result<Self> {
        let module_id = ser_module
            .first()
            .map(|&id| RuntimeModuleId::try_from(id))
            .transpose()?;

        match module_id {
            Some(RuntimeModuleId::ASModuleId) => Ok(RuntimeModule::ASModule(
                ASModule::deserialize(&ser_module[1..], limit, gas_costs, condom_limits)?,
            )),
            Some(RuntimeModuleId::WasmV1ModuleId) => Ok(RuntimeModule::WasmV1Module(
                WasmV1Module::deserialize(&ser_module[1..], limit, gas_costs, condom_limits)?,
            )),
            None => Err(anyhow!("Empty bytecode")),
        }
    }

    /// Check the exports of a compiled module to see if it contains the given
    /// function
    pub(crate) fn function_exists(&self, function: &str) -> bool {
        match self {
            RuntimeModule::ASModule(module) => module.function_exists(function),
            RuntimeModule::WasmV1Module(module) => module.function_exists(function),
        }
    }
}

/// Select and launch the adequate execution function
pub(crate) fn exec(
    interface: &dyn Interface,
    rt_module: RuntimeModule,
    function: &str,
    param: &[u8],
    limit: u64,
    gas_costs: GasCosts,
    condom_limits: CondomLimits,
) -> VMResult<(Response, Option<GasCalibrationResult>)> {
    let response = match rt_module {
        RuntimeModule::ASModule(module) => exec_as_module(
            interface,
            module,
            function,
            param,
            limit,
            gas_costs,
            condom_limits,
        )?,
        RuntimeModule::WasmV1Module(module) => exec_wasmv1_module(
            interface,
            module,
            function,
            param,
            limit,
            gas_costs,
            condom_limits,
        )
        .map_err(|err| {
            VMError::InstanceError(format!("Failed to execute WasmV1 module: {}", err))
        })?,
    };
    Ok(response)
}

/// Library Input, take a `module` wasm built with the massa environment,
/// must have a main function inside written in AssemblyScript:
///
/// ```js
/// import { print } from "massa-sc-std";
///
/// export function main(_args: string): i32 {
///     print("hello world");
///     return 0;
/// }
/// ```
/// Return:
/// the remaining gas.
pub fn run_main(
    interface: &dyn Interface,
    rt_module: RuntimeModule,
    limit: u64,
    gas_costs: GasCosts,
    condom_limits: CondomLimits,
) -> VMResult<Response> {
    Ok(exec(
        interface,
        rt_module,
        settings::MAIN,
        b"",
        limit,
        gas_costs,
        condom_limits,
    )?
    .0)
}

/// Library Input, take a `module` wasm built with the massa environment,
/// run a function of that module with the given parameter:
///
/// ```js
/// import { print } from "massa-sc-std";
///
/// export function hello_world(_args: string): i32 {
///     print("hello world");
///     return 0;
/// }
/// ```
pub fn run_function(
    interface: &dyn Interface,
    rt_module: RuntimeModule,
    function: &str,
    param: &[u8],
    limit: u64,
    gas_costs: GasCosts,
    condom_limits: CondomLimits,
) -> VMResult<Response> {
    Ok(exec(
        interface,
        rt_module,
        function,
        param,
        limit,
        gas_costs,
        condom_limits,
    )?
    .0)
}

/// Same as run_main but return a GasCalibrationResult
#[cfg(feature = "gas_calibration")]
pub fn run_main_gc(
    interface: &dyn Interface,
    rt_module: RuntimeModule,
    param: &[u8],
    limit: u64,
    gas_costs: GasCosts,
    condom_limits: CondomLimits,
) -> VMResult<GasCalibrationResult> {
    Ok(exec(
        interface,
        rt_module,
        settings::MAIN,
        param,
        limit,
        gas_costs,
        condom_limits,
    )?
    .1
    .unwrap())
}

// tests for serialize and deserialize
#[test]
fn test_serialize_deserialize() {
    let bytecode = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/dummy.wat"));

    // ASModule
    {
        let module = RuntimeModule::ASModule(
            ASModule::new(
                bytecode,
                0,
                GasCosts::default(),
                Compiler::CL,
                CondomLimits::default(),
            )
            .unwrap(),
        );

        let serialized = module.serialize().unwrap();
        assert_eq!(
            *serialized.first().unwrap(),
            RuntimeModuleId::ASModuleId as u8
        );

        let serialized2 = RuntimeModule::deserialize(
            &serialized,
            0,
            GasCosts::default(),
            CondomLimits::default(),
        )
        .unwrap()
        .serialize()
        .unwrap();

        assert_eq!(serialized, serialized2);
    }

    // WasmV1Module
    {
        let module = RuntimeModule::WasmV1Module(
            WasmV1Module::compile(
                bytecode,
                0,
                GasCosts::default(),
                Compiler::CL,
                CondomLimits::default(),
            )
            .unwrap(),
        );

        let serialized = module.serialize().unwrap();
        assert_eq!(
            *serialized.first().unwrap(),
            RuntimeModuleId::WasmV1ModuleId as u8
        );

        let serialized2 = RuntimeModule::deserialize(
            &serialized,
            0,
            GasCosts::default(),
            CondomLimits::default(),
        )
        .unwrap()
        .serialize()
        .unwrap();

        assert_eq!(serialized, serialized2);
    }
}
