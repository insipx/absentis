use failure::*;
use crate::json_builder::JsonBuildError;

#[derive(Debug, Fail)]
pub enum RpcError {
    #[fail(display = "TLS Connection Error")]
    TlsConnectionError(#[fail(cause)] hyper_tls::Error),
    #[fail(display = "Failed to parse URI from parts")]
    FromPartsUrlParseError(#[fail(cause)] http::uri::InvalidUriParts),
    #[fail(display = "Failed to parse URI")]
    UrlParseError(#[fail(cause)] http::uri::InvalidUri),
    #[fail(display = "Failed to serialize into JSON")] 
    JsonSerializeError(#[fail(cause)] serde_json::error::Error),
    #[fail(display = "Invalid Header Value for in HTTP Request")]
    InvalidHeaderValue,
    // #[fail(display = "Tokio BlockError")]
    // BlockError(#[fail(cause)] tokio::executor::current_thread::BlockError<hyper::error::Error>),
    #[fail(display = "Json Build Error")]
    JsonError(#[fail(cause)] JsonBuildError),
    #[fail(display = "A Networking Error Occured")]
    Net(#[fail(cause)] hyper::error::Error)
}

impl From<hyper::error::Error> for RpcError {
    fn from(err: hyper::error::Error) -> RpcError {
        RpcError::Net(err)
    }
}

impl From<JsonBuildError> for RpcError {
    fn from(err: JsonBuildError) -> RpcError {
        RpcError::JsonError(err)
    }
}

impl From<hyper_tls::Error> for RpcError {
    fn from(err: hyper_tls::Error) -> RpcError {
        RpcError::TlsConnectionError(err)
    }
}

impl From<http::uri::InvalidUriParts> for RpcError {
    fn from(err: http::uri::InvalidUriParts) -> RpcError {
        RpcError::FromPartsUrlParseError(err)
    }
}

impl From<http::uri::InvalidUri> for RpcError {
    fn from(err: http::uri::InvalidUri) -> RpcError {
        RpcError::UrlParseError(err)
    }
}

impl From<serde_json::error::Error> for RpcError {
    fn from(err: serde_json::error::Error) -> RpcError {
        RpcError::JsonSerializeError(err)
    }
}
