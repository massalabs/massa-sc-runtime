use massa_proto_rs::massa::model::v1::{
    NativeAddress, NativeAmount, NativeHash, NativePubKey, NativeSig,
};

use crate::wasmv1_execution::WasmV1Error;

pub(crate) trait TryInto<T> {
    fn try_into(&self) -> Result<T, WasmV1Error>;
}

pub(crate) trait Check {
    fn is_valid(&self) -> Result<bool, WasmV1Error>;
}

impl TryInto<String> for NativeAddress {
    fn try_into(&self) -> Result<String, WasmV1Error> {
        String::from_utf8(self.content.clone()).map_err(|err| {
            WasmV1Error::RuntimeError(format!(
                "Could not convert address to string: {}",
                err
            ))
        })
    }
}

impl TryInto<String> for NativePubKey {
    fn try_into(&self) -> Result<String, WasmV1Error> {
        String::from_utf8(self.content.clone()).map_err(|err| {
            WasmV1Error::RuntimeError(format!(
                "Could not convert pubkey to string: {}",
                err
            ))
        })
    }
}

impl TryInto<String> for NativeSig {
    fn try_into(&self) -> Result<String, WasmV1Error> {
        String::from_utf8(self.content.clone()).map_err(|err| {
            WasmV1Error::RuntimeError(format!(
                "Could not convert sig to string: {}",
                err
            ))
        })
    }
}

impl TryInto<String> for NativeHash {
    fn try_into(&self) -> Result<String, WasmV1Error> {
        String::from_utf8(self.content.clone()).map_err(|err| {
            WasmV1Error::RuntimeError(format!(
                "Could not convert hash to string: {}",
                err
            ))
        })
    }
}

impl TryInto<NativeAddress> for String {
    fn try_into(&self) -> Result<NativeAddress, WasmV1Error> {
        Ok(NativeAddress {
            category: todo!(),
            version: todo!(),
            content: todo!(),
        })
    }
}

impl TryInto<NativePubKey> for String {
    fn try_into(&self) -> Result<NativePubKey, WasmV1Error> {
        Ok(NativePubKey {
            version: todo!(),
            content: todo!(),
        })
    }
}

impl TryInto<NativeSig> for String {
    fn try_into(&self) -> Result<NativeSig, WasmV1Error> {
        Ok(NativeSig {
            version: todo!(),
            content: todo!(),
        })
    }
}

impl TryInto<NativeHash> for String {
    fn try_into(&self) -> Result<NativeHash, WasmV1Error> {
        Ok(NativeHash {
            version: todo!(),
            content: todo!(),
        })
    }
}

impl Check for NativeAddress {
    fn is_valid(&self) -> Result<bool, WasmV1Error> {
        Ok(todo!())
    }
}

impl Check for NativePubKey {
    fn is_valid(&self) -> Result<bool, WasmV1Error> {
        Ok(todo!())
    }
}

impl Check for NativeSig {
    fn is_valid(&self) -> Result<bool, WasmV1Error> {
        Ok(todo!())
    }
}

impl Check for NativeHash {
    fn is_valid(&self) -> Result<bool, WasmV1Error> {
        Ok(todo!())
    }
}

// TODO: this is a temporary implementation, need to manage denum
impl TryInto<u64> for NativeAmount {
    fn try_into(&self) -> Result<u64, WasmV1Error> {
        Ok(self.mantissa)
    }
}
