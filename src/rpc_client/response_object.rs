//! Object holding possible responses from JSON-RPC's (Parity, Infura, Geth, etc)
use log::{debug, log, info};
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
    EthGasPrice(Hex), // eth_gasPrice
    EthGetBalance(Hex), // eth_getBalance
    Nil, // no response
}

impl PartialEq for ResponseObject {
    fn eq(&self, other: &ResponseObject) -> bool {
        self.to_str() == other.to_str()
    }
}

macro_rules! parse_call_result {
    (string, $call:ident, $val: ident) => ({
        Some(ApiCall::$call) => {
            if !$val.is_string() {
                mismatched_types!("String", $val)
            } else {
                let hex = serde_json::from_str(&val.take().to_string());
                Ok(ResponseObject::$call(verb_err!(hex)))
            }
        }
    });
    (block, $call:ident) => ({
        Some(ApiCall::$call) => {
            if !$val.is_object() {
                mismatched_types!("Map", val)
            } else {
                debug!("Map(BLOCK) String: {}", val.to_string().yellow().bold());
                let block = serde_json::from_str(&val.take().to_string());
                Ok(ResponseObject::EthGetBlockByNumber(verb_err!(block)))
            }
        }
    });
    (tx, $call:ident) => ({
        unimplemented!();
    });
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
            Some(ApiCall::EthGasPrice) => {
                if !val.is_string() {
                    mismatched_types!("String", val)
                } else {
                    let hex = serde_json::from_str(&val.take().to_string());
                    Ok(ResponseObject::EthGasPrice(verb_err!(hex)))
                }
            },
            Some(ApiCall::EthGetBalance) => {
                if !val.is_string() {
                    mismatched_types!("String", val)
                } else {
                    let hex = serde_json::from_str(&val.take().to_string());
                    Ok(ResponseObject::EthGetBalance(verb_err!(hex)))
                }
            },
            Some(ApiCall::Nil) => {
                Ok(ResponseObject::Nil)
            },
            _=> panic!("Resposne does not exist {}", err_loc!())
        }
    }
    
    pub fn from_bytes(body: bytes::Bytes) -> std::result::Result<Self, JsonBuildError> {
        let json: JsonRpcObject = serde_json::from_slice(&body.to_vec())?;
        if json.is_error() {
            info!("JsonRpcObject: {}", json);
            let err_info = json.err_info().unwrap();
            return Err(JsonBuildError::RPCError(err_info.0, err_info.1))
        }
        Ok(json.get_result())
    }

    pub fn to_str(&self) -> String {
        match self {
            ResponseObject::EthBlockNumber(_) => "EthBlockNumber".to_owned(),
            ResponseObject::EthGetBlockByNumber(_) => "EthGetBlockByNumber".to_owned(),
            ResponseObject::EthGasPrice(_) => "EthGasPrice".to_owned(),
            ResponseObject::EthGetBalance(_) => "EthGetBalance".to_owned(),
            ResponseObject::Nil => "Nil".to_owned(),
        }
    }
}
