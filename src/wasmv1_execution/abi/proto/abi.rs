#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Address {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Amount {
    #[prost(fixed64, tag = "1")]
    pub amount: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}
/// CreateSC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateScRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub bytecode: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateScResponse {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<Address>,
}
/// CallSC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallRequest {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<Address>,
    #[prost(string, tag = "2")]
    pub function: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub arg: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub call_coins: ::core::option::Option<Amount>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub return_data: ::prost::alloc::vec::Vec<u8>,
}
/// LocalCall
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LocalCallRequest {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<Address>,
    #[prost(string, tag = "2")]
    pub function: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub arg: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LocalCallResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub return_data: ::prost::alloc::vec::Vec<u8>,
}
/// GenerateEvent
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenerateEventRequest {
    #[prost(string, tag = "1")]
    pub event: ::prost::alloc::string::String,
}
/// TransferCoins
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransferCoinsRequest {
    #[prost(message, optional, tag = "1")]
    pub to_address: ::core::option::Option<Address>,
    #[prost(message, optional, tag = "2")]
    pub raw_amount: ::core::option::Option<Amount>,
}
/// FunctionExists
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FunctionExistsRequest {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<Address>,
    #[prost(string, tag = "2")]
    pub function: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FunctionExistsResponse {
    #[prost(bool, tag = "1")]
    pub exists: bool,
}
