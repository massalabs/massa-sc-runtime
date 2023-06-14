use massa_proto_rs::massa::model::v1::{NativeAddress, NativeAmount};

use super::WasmV1Error;

pub(crate) trait TryToString {
    fn try_to_string(&self) -> Result<String, WasmV1Error>;
}

impl TryToString for NativeAddress {
    fn try_to_string(&self) -> Result<String, WasmV1Error> {
        String::from_utf8(self.content.clone()).map_err(|err| {
            WasmV1Error::RuntimeError(format!(
                "Could not convert address to string: {}",
                err
            ))
        })
    }
}

pub(crate) trait TryToU64 {
    fn try_to_u64(&self) -> Result<u64, WasmV1Error>;
}

// TODO: this is a temporary implementation, need to manage denum
impl TryToU64 for NativeAmount {
    fn try_to_u64(&self) -> Result<u64, WasmV1Error> {
        Ok(self.mantissa)
    }
}
