/// Address
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Address {
    /// Address is a string representation of the address
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
/// Amount
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Amount {
    /// Amount is a 64-bit unsigned integer
    #[prost(fixed64, tag = "1")]
    pub amount: u64,
}
/// Empty
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}
/// CreateSC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateScRequest {
    /// Bytecode is the compiled code of the smart contract
    #[prost(bytes = "vec", tag = "1")]
    pub bytecode: ::prost::alloc::vec::Vec<u8>,
}
///   CreateSCResponse
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateScResponse {
    /// Address is a string representation of the address
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<Address>,
}
/// CallSC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallRequest {
    /// Address is the address of the smart contract
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<Address>,
    /// Function is the name of the function to call
    #[prost(string, tag = "2")]
    pub function: ::prost::alloc::string::String,
    /// Arg is the argument to the function
    #[prost(bytes = "vec", tag = "3")]
    pub arg: ::prost::alloc::vec::Vec<u8>,
    /// call_coins is the amount of coins to pay for the call
    #[prost(message, optional, tag = "4")]
    pub call_coins: ::core::option::Option<Amount>,
}
/// CallResponse
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallResponse {
    /// Return_data is the return value of the function
    #[prost(bytes = "vec", tag = "1")]
    pub return_data: ::prost::alloc::vec::Vec<u8>,
}
/// LocalCall
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LocalCallRequest {
    /// Address is the address of the smart contract
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<Address>,
    /// Function is the name of the function to call
    #[prost(string, tag = "2")]
    pub function: ::prost::alloc::string::String,
    /// Arg is the argument to the function
    #[prost(bytes = "vec", tag = "3")]
    pub arg: ::prost::alloc::vec::Vec<u8>,
}
/// LocalCallResponse
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LocalCallResponse {
    /// Return_data is the return value of the function
    #[prost(bytes = "vec", tag = "1")]
    pub return_data: ::prost::alloc::vec::Vec<u8>,
}
/// GenerateEventRequest
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenerateEventRequest {
    /// Event
    #[prost(string, tag = "1")]
    pub event: ::prost::alloc::string::String,
}
/// TransferCoins
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransferCoinsRequest {
    /// To_address is the address to transfer coins to
    #[prost(message, optional, tag = "1")]
    pub to_address: ::core::option::Option<Address>,
    /// Amount is the amount of coins to transfer
    #[prost(message, optional, tag = "2")]
    pub raw_amount: ::core::option::Option<Amount>,
}
/// FunctionExists
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FunctionExistsRequest {
    /// Address is the address of the smart contract
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<Address>,
    /// Function is the name of the function to call
    #[prost(string, tag = "2")]
    pub function: ::prost::alloc::string::String,
}
/// FunctionExistsResponse
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FunctionExistsResponse {
    /// Exists is true if the function exists
    #[prost(bool, tag = "1")]
    pub exists: bool,
}
