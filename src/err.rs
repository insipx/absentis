use failure::*;

#[derive(Debug, Fail)]
pub enum NodeError {
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
    #[fail(display = "Tokio BlockError")]
    BlockError(#[fail(cause)] tokio::executor::current_thread::BlockError<hyper::error::Error>),
}

impl From<hyper_tls::Error> for NodeError {
    fn from(err: hyper_tls::Error) -> NodeError {
        NodeError::TlsConnectionError(err)
    }
}

impl From<http::uri::InvalidUriParts> for NodeError {
    fn from(err: http::uri::InvalidUriParts) -> NodeError {
        NodeError::FromPartsUrlParseError(err)
    }
}

impl From<http::uri::InvalidUri> for NodeError {
    fn from(err: http::uri::InvalidUri) -> NodeError {
        NodeError::UrlParseError(err)
    }
}

impl From<serde_json::error::Error> for NodeError {
    fn from(err: serde_json::error::Error) -> NodeError {
        NodeError::JsonSerializeError(err)
    }
}

