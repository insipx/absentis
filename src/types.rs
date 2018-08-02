use failure::*;
use std::str::FromStr;

pub const MAINNET_AUTHORITY: &'static str = "https://mainnet.infura.io/";
pub const JSON_RPC_VERSION: &'static str = "2.0";
pub const JSON_APP_HEADER: &'static str = "application/json";

#[derive(Clone, Debug)]
pub struct Uri(hyper::Uri);

pub enum ApiCall {
  EthBlockNumber, // eth_blockNumber
  EthGetBlockByNumber, // eth_getBlockByNumber
}

impl ApiCall {

    pub fn from_id(id: usize) -> Self {
        match id {
            1 => ApiCall::EthBlockNumber,
            2 => ApiCall::EthGetBlockByNumber,
            _ => panic!("No Id for API call found!")
        }
    }

    pub fn method_info(&self) -> (usize, String) {
        match self {
            EthBlockNumber => (1, "eth_blockNumber".to_string()),
            EthGetBlockByNumber => (2, "eth_getBlockByNumber".to_string()),
        }
    }
}

impl From<usize> for ApiCall {
    fn from(call: usize) -> ApiCall {
        match call {
            1 => ApiCall::EthBlockNumber,
            2 => ApiCall::EthGetBlockByNumber,
            _ => panic!("No Id for API call found!")
        }
    }
}
// String conversions should really not be used in production, they are for tests
// TODO: Convert into errors
impl From<Uri> for String {
    fn from(uri: Uri) -> String {
        let parts = uri.0.into_parts();
        let scheme = parts.scheme.expect("Couldn't Parse URI, Invalid Scheme");
        let authority = parts.authority.expect("Couldn't parse URI, Invalid Authority");
        let path_and_query = parts.path_and_query.expect("Couldn't parse URI, Invalid Path & Query");
        format!("{}://{}{}", scheme, authority, path_and_query).to_string()
    }
}

impl From<Uri> for hyper::Uri {
    fn from(uri: Uri) -> hyper::Uri {
        uri.0
    }
}

impl From<hyper::Uri> for Uri {
    fn from(uri: hyper::Uri) -> Uri {
        Uri(uri)
    }
}

impl From<&str> for Uri {
    fn from(str: &str) -> Uri {
        let shared = bytes::Bytes::from(str);
        let uri: hyper::Uri = hyper::Uri::from_shared(shared).expect("Could not convert from bytes to URI");
        Uri(uri)
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Could not convert str to URI")]
pub struct UriConversionError;

impl FromStr for Uri {
    type Err = UriConversionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Uri::from(s))
    }
}



