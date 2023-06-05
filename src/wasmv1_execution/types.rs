use massa_proto::massa::abi::v1::{NativeAddress, NativeAmount};

// ****************************************************************************
// Simple wrapper so we can redefine traits
pub(crate) struct Address(NativeAddress);

impl From<NativeAddress> for Address {
    fn from(value: NativeAddress) -> Self {
        Address(value)
    }
}

impl Into<NativeAddress> for Address {
    fn into(self) -> NativeAddress {
        self.0
    }
}

impl TryInto<String> for Address {
    type Error = String;

    fn try_into(self) -> Result<String, Self::Error> {
        String::from_utf8(self.0.content)
            .map_err(|err| format!("Could not convert address to string: {}", err))
    }
}

// ****************************************************************************
// Simple wrapper so we can redefine traits

#[derive(Clone)]
pub struct Amount(NativeAmount);

impl From<NativeAmount> for Amount {
    fn from(value: NativeAmount) -> Self {
        Amount(value)
    }
}

impl Into<NativeAmount> for Amount {
    fn into(self) -> NativeAmount {
        self.0
    }
}

impl TryInto<u64> for Amount {
    type Error = String;

    // FIXME assume for now that the amount is in the numerator i.e. denominator is always 1
    fn try_into(self) -> Result<u64, Self::Error> {
        Ok(self.0.numerator)
    }
}

// impl TryInto<String> for Amount {
//     type Error = String;

//     fn try_into(self) -> Result<String, Self::Error> {
//         native_amount_to_string(self.0.clone())
//     }
// }

// impl TryFrom<String> for Amount {
//     type Error = String;

//     fn try_from(value: String) -> Result<Self, Self::Error> {
//         Ok((value)?.into())
//     }
// }
