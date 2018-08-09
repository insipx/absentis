use failure::*;
use serde_derive::Deserialize;
use super::request_object::RequestBuildError;

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
    JsonError(#[fail(cause)]  RequestBuildError),
    #[fail(display = "A Networking Error Occured")]
    Net(#[fail(cause)] hyper::error::Error),
    #[fail(display = "Missing parameter: {}", _0)]
    MissingParameter(String)
}

impl From<hyper::error::Error> for RpcError {
    fn from(err: hyper::error::Error) -> RpcError {
        RpcError::Net(err)
    }
}

impl From<RequestBuildError> for RpcError {
    fn from(err: RequestBuildError) -> RpcError {
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

#[derive(Fail, Debug)]
pub struct TypeMismatchError {
    expected: String,
    got: String
}

impl TypeMismatchError {
    crate fn new(expected: String, got: String) -> Self {
        TypeMismatchError {
            expected, got,
        }
    }
}

impl std::fmt::Display for TypeMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "expected type: {}, got: {}", self.expected, self.got)
    }
}

#[derive(Debug, Fail)]
pub enum ResponseBuildError {
    #[fail(display = "Error Deserializing JSON `Result`: {}", _0)]
    SerializationError(#[cause] serde_json::error::Error),
    #[fail(display = "Error building Json Response Object {}: ", _0)]
    HyperError(#[cause] hyper::error::Error),
    #[fail(display = "Mismatched types during build: {}", _0)]
    MismatchedTypes(TypeMismatchError),
    #[fail(display = "The Ethereum JsonRPC returned an error: {}, code: {}", _0, _1)]
    RPCError(String, i64)
}

impl From<serde_json::error::Error> for ResponseBuildError {
    fn from(err: serde_json::error::Error) -> Self {
        ResponseBuildError::SerializationError(err)
    }
}

impl From<hyper::error::Error> for ResponseBuildError {
    fn from(err: hyper::error::Error) -> ResponseBuildError {
        ResponseBuildError::HyperError(err)
    }
}

#[derive(Deserialize, Debug)]
pub struct JsonRpcError  {
    code: i64,
    message: String,
    data: Option<serde_json::Value>
}

impl JsonRpcError {
    pub fn info(&self) -> (String, i64) {
        (self.message.clone(), self.code)
    }
}


