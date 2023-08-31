use wasmer::{AsStoreMut, AsStoreRef, Instance, Memory, TypedFunction};

use super::WasmV1Error;

#[derive(Clone)]
pub struct Ffi {
    guest_alloc_func: TypedFunction<i32, i32>,
    guest_dealloc_func: Option<TypedFunction<i32, ()>>,
    guest_memory: Memory,
}

impl Ffi {
    pub fn try_new(instance: &Instance, store: &impl AsStoreRef) -> Result<Self, WasmV1Error> {
        let guest_alloc_func = instance
            .exports
            .get_typed_function::<i32, i32>(store, "__alloc")
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!(
                    "Could not get __alloc function from guest instance: {}",
                    err
                ))
            })?;
        let guest_dealloc_func = match instance
            .exports
            .get_typed_function::<i32, ()>(store, "__dealloc")
        {
            // deallocator was successfully loaded
            Ok(func) => Some(func),
            // deallocator is absent
            Err(wasmer::ExportError::Missing(_)) => None,
            // deallocator is present but has the wrong signature
            Err(err) => {
                return Err(WasmV1Error::RuntimeError(format!(
                    "Invalid __dealloc function signature exported by guest instance: {}",
                    err
                )));
            }
        };
        let guest_memory = instance
            .exports
            .get_memory("memory")
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!(
                    "Could not get memory from guest instance: {}",
                    err
                ))
            })?
            .clone();
        Ok(Self {
            guest_alloc_func,
            guest_dealloc_func,
            guest_memory,
        })
    }

    /// Get the memory maximum size in bytes
    pub fn get_max_mem_size(&self, store: &impl AsStoreRef) -> u64 {
        self.guest_memory.view(store).data_size()
    }

    /// Reads a buffer and tries to deallocate it guest-side.
    /// Assumes memory layout is: [len: i32 little-endian][data: u8*]
    pub fn take_buffer(
        &self,
        store: &mut impl AsStoreMut,
        offset: i32,
    ) -> Result<Vec<u8>, WasmV1Error> {
        let Ok(offset_u64): Result<u64, _> = offset.try_into() else {
            return Err(WasmV1Error::RuntimeError(format!(
                "Invalid memory read offset: {}",
                offset
            )));
        };
        let view = self.guest_memory.view(store);

        let mut len_buffer = [0u8; 4];
        view.read(offset_u64, &mut len_buffer).map_err(|err| {
            WasmV1Error::RuntimeError(format!(
                "Could not read length prefix from guest memory: {}",
                err
            ))
        })?;
        let len = i32::from_le_bytes(len_buffer);
        let Ok(len): Result<u64, _> = len.try_into() else {
            return Err(WasmV1Error::RuntimeError(format!(
                "Memory read length invalid: {}",
                len
            )));
        };

        let memory_size = view.data_size();
        if len > memory_size {
            return Err(WasmV1Error::RuntimeError(format!(
                "Memory read length exceeds memory size: {}",
                len
            )));
        }
        let mut buffer = vec![
            0u8;
            len.try_into().expect(
                "Buffer too large to be addressed on this system using usize"
            )
        ];
        let Some(data_offset) = offset_u64.checked_add(len_buffer.len() as u64) else {
            return Err(WasmV1Error::RuntimeError("Offset overflow".into()));
        };
        view.read(data_offset, &mut buffer).map_err(|err| {
            WasmV1Error::RuntimeError(format!("Could not read guest memory: {}", err))
        })?;

        // Deallocate the buffer if there is a dealloc guest function
        if let Some(guest_dealloc_func) = &self.guest_dealloc_func {
            guest_dealloc_func.call(store, offset).map_err(|err| {
                WasmV1Error::RuntimeError(format!("__dealloc function call failed: {}", err))
            })?;
        }
        Ok(buffer)
    }

    /// Allocates and creates a buffer.
    pub fn create_buffer(
        &self,
        store: &mut impl AsStoreMut,
        buffer: &[u8],
    ) -> Result<i32, WasmV1Error> {
        let len: i32 = buffer.len().try_into().map_err(|err| {
            WasmV1Error::RuntimeError(format!("Could not convert buffer length to i32: {}", err))
        })?;
        let offset: i32 = self.guest_alloc_func.call(store, len).map_err(|err| {
            WasmV1Error::RuntimeError(format!("__alloc function call failed: {}", err))
        })?;
        let Ok(offset_u64): Result<u64, _> = offset.try_into() else {
            return Err(WasmV1Error::RuntimeError(format!(
                "__alloc returned invalid pointer: {}",
                offset
            )));
        };

        self.guest_memory
            .view(store)
            .write(offset_u64, buffer)
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!(
                    "Could not write in allocated guest memory: {}",
                    err
                ))
            })?;
        Ok(offset)
    }
}
