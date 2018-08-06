//! An object that represents the JSON Data Responses and Requests to JsonRPC's
use log::{log, debug, info};
use failure::{Fail};
use serde_derive::*;
use std::fmt;
use serde::de::{self, Deserializer, Deserialize, Visitor, MapAccess};
use colored::Colorize;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::types::JSON_RPC_VERSION;
use super::api_call::ApiCall;
use super::response_object::ResponseObject;

#[derive(Serialize, Debug)]
pub struct RequestObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    params: Vec<serde_json::Value>,
}

impl std::fmt::Display for RequestObject {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\n\t{}: {:?},\n \t{}: {:?},\n \t{}: {:?},\n \t{}: {:?}\n", 
               "id".bright_white().bold().underline(), self.id, 
               "jsonrpc".bright_white().bold().underline(), self.jsonrpc, 
               "method".bright_white().bold().underline(), self.method,
               "params".bright_white().bold().underline(), self.params)
    }
}


#[derive(Fail, Debug)]
pub enum RequestBuildError {
    #[fail(display = "Error Serializing JSON Object: {}", _0)]
    SerializationError(#[cause] serde_json::error::Error),
    #[fail(display = "Hyper Error while building Json Response Object: {}", _0)]
    HyperError(#[cause] hyper::error::Error),
}

impl From<hyper::error::Error> for RequestBuildError {
    fn from(err: hyper::error::Error) -> RequestBuildError {
        RequestBuildError::HyperError(err)
    }
}

impl From<serde_json::error::Error> for RequestBuildError {
    fn from(err: serde_json::error::Error) -> RequestBuildError {
        RequestBuildError::SerializationError(err)
    }
}

impl Default for RequestObject {
    fn default() -> RequestObject {
        RequestObject {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id: None,
            method: None,
            params: Vec::new(),
        } 
    }
}

impl RequestObject {

    pub fn method(&mut self, val: ApiCall) -> &mut Self {
        let new = self;
        new.id = Some(val.to_str());
        new.method = Some(val.method());
        new
    }

    pub fn params<T: Into<Vec<serde_json::Value>>>(&mut self, val: T) -> &mut Self {
        let new = self;
        new.params = val.into();
        new
    }

    pub fn build(&self) -> std::result::Result<String, RequestBuildError> {
        info!("\n{}: {}", "Json Rpc Object (SEND)".bright_green().underline(), self);
        info!("\n{}: \n{:?}", "Json Rpc Object (SEND, SERIALIZED)", serde_json::to_string(self));
        Ok(serde_json::to_string(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{log, info, error, debug};
    use serde_json::{json, json_internal};
    use env_logger;
    #[test]
    fn it_should_create_json() {
        env_logger::try_init();
        let test = json!({
           "id": "EthBlockNumber",
           "jsonrpc": "2.0",
           "method": "eth_blockNumber",
           "params": [],
        }).to_string();

        let json = RequestObject::default().method(ApiCall::EthBlockNumber).build().expect("Bulding JSON failed");
        info!("{}:{:?}", "JSON OBJECT".cyan().bold(), json);
        assert_eq!(test, json);
    }
}
