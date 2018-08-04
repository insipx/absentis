//! Object holding possible responses from JSON-RPC's (Parity, Infura, Geth, etc)
use log::{debug, log};
use serde_derive::{Deserialize};
use colored::Colorize;
use num_traits::FromPrimitive;

use crate::ethereum_objects::{Hex, Block};
use super::err::{ResponseBuildError, TypeMismatchError};
use super::jsonrpc_object::{JsonRpcObject, JsonBuildError};
use super::api_call::ApiCall;

#[derive(Debug, Deserialize)]
pub enum ResponseObject {
    EthBlockNumber(Hex), // eth_blockNumber
    EthGetBlockByNumber(Block), //eth_getBlockByNumber
    Nil, // no response
}

impl ResponseObject {
    pub fn new(body: String) -> std::result::Result<Self, ResponseBuildError> {
        debug!("{}: {:#?}", "JSON Response Result Object".cyan(), body.yellow());
        let json: JsonRpcObject = serde_json::from_str(&body)?;
        Ok(json.get_result())
    }
    
    // parses a serde_json::Value into a ResponseObject
    // Value must be a Value::String or Value::Object
    pub fn from_serde_value(mut val: serde_json::Value, id: usize) -> Result<Self, ResponseBuildError> {
        match ApiCall::from_usize(id) {
            Some(ApiCall::EthBlockNumber) => {
                if !val.is_string() {   
                    mismatched_types!("String", val)
                } else {
                    let hex = serde_json::from_str(&val.take().to_string());
                    Ok(ResponseObject::EthBlockNumber(verb_err!(hex)))
                }
            },
            Some(ApiCall::EthGetBlockByNumber) => {
                if !val.is_object() {
                    mismatched_types!("Map", val)
                } else {
                    debug!("Map String: {}", val.to_string().yellow().bold());
                    let block = serde_json::from_str(&val.take().to_string());
                    Ok(ResponseObject::EthGetBlockByNumber(verb_err!(block)))
                }
            },
            Some(ApiCall::Nil) => {
                Ok(ResponseObject::Nil)
            },
            _=> panic!("Resposne does not exist {}", err_loc!())
        }
    }
    
    pub fn from_bytes(body: bytes::Bytes) -> std::result::Result<Self, JsonBuildError> {
        // debug!("{}: {}", "JSON Response Result Object".cyan().bold(), std::str::from_utf8(&*body).unwrap().yellow().bold());
        // debug!("In Function {} in file {}; line: {}", "`from_bytes`".bold().underline().bright_cyan(), file!().bold().underline(), line!().to_string().bold().bright_white().underline());
        let json: JsonRpcObject = serde_json::from_slice(&body.to_vec())?;
        // debug!("{}: {:?}", "JSON Response Object, Deserialized".cyan().bold(), json);
        Ok(json.get_result())
    }

    pub fn to_str(&self) -> String {
        match self {
            ResponseObject::EthBlockNumber(_) => "EthBlockNumber".to_owned(),
            ResponseObject::EthGetBlockByNumber(_) => "EthGetBlockByNumber".to_owned(),
            ResponseObject::Nil => "Nil".to_owned(),
        }
    }
}
