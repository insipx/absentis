use failure::{Fail, Error as FError};
use serde_derive::*;
use serde_json::{self, from_str, from_slice, Error as JError, json, json_internal};
use std::error::Error;
use std::fmt;
use serde::de::{self, Deserializer, Deserialize, Visitor, SeqAccess, MapAccess};
use colored::Colorize;
use log::*;
use crate::types::*;
use crate::ethereum_objects::{ResponseObject};

#[derive(Serialize, Debug)]
pub struct JsonBuilder {
    id: usize,
    jsonrpc: String,
    #[serde(skip_serializing)]
    result: ResponseObject,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    params: Vec<serde_json::Value>,
}


#[derive(Fail, Debug)]
pub enum JsonBuildError {
    #[fail(display = "Error building JSON Object")]
    SerializationError(#[fail(cause)] serde_json::error::Error),
    #[fail(display = "Hyper Error while building Json Response Object")]
    HyperError(#[fail(cause)] hyper::error::Error)
}

impl From<hyper::error::Error> for JsonBuildError {
    fn from(err: hyper::error::Error) -> JsonBuildError {
        JsonBuildError::HyperError(err)
    }
}

impl From<serde_json::error::Error> for JsonBuildError {
    fn from(err: serde_json::error::Error) -> JsonBuildError {
        JsonBuildError::SerializationError(err)
    }
}

impl Default for JsonBuilder {
    fn default() -> JsonBuilder {
        JsonBuilder {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id: 0,
            result: ResponseObject::Nil,
            method: None,
            params: Vec::new(),
        } 
    }
}

impl JsonBuilder {

    pub fn method(&mut self, val: ApiCall) -> &mut Self {
        let new = self;
        let (id, method) = val.method_info();
        new.id = id;
        new.method = Some(method.into());
        new
    }

    pub fn params<T: Into<Vec<serde_json::Value>>>(&mut self, val: T) -> &mut Self {
        let new = self;
        new.params = val.into();
        new
    }

    pub fn build(&self) -> std::result::Result<String, JsonBuildError> {
        debug!("{}: {:?}","JSON Response Object".cyan().bold(), self);
        debug!("{}: {:?}", "JSON Object, SERIALIZED".yellow().bold(), serde_json::to_string(self)?);
        Ok(serde_json::to_string(self)?)
    }
}

impl JsonBuilder {
    crate fn get_id(&self) -> usize {
        self.id
    }

    // returns a raw string literal of the result
    crate fn get_result(self) -> ResponseObject {
        self.result
    }
}


impl<'de> Deserialize<'de> for JsonBuilder {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
        
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {Id, JsonRpc, Result, Method, Params};

        struct JsonBuilderVisitor;
        impl<'de> Visitor<'de> for JsonBuilderVisitor {
            type Value = JsonBuilder;
    
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct JsonBuilder")
            }
            
            fn visit_map<V>(self, mut map: V) -> std::result::Result<JsonBuilder, V::Error> 
                where
                    V: MapAccess<'de>
            {
                let mut id = None;
                let mut jsonrpc = None;
                let mut result: Option<String> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        },
                        Field::JsonRpc => {
                            if jsonrpc.is_some() { 
                                return Err(de::Error::duplicate_field("jsonrpc"));
                            }
                            jsonrpc = Some(map.next_value()?);
                        },
                        Field::Result => {
                            if result.is_some() { 
                                return Err(de::Error::duplicate_field("result"));
                            }
                            result = Some(map.next_value()?);
                        },
                        Field::Method => {
                            /* return Err(de::Error::unknown_field("Don't deserialize 'Method'", map.next_value()?));*/
                        },
                        Field::Params => {
                            /* return Err(de::Error::unknown_field("Don't deserialize 'Params'", map.next_value()?)); */
                        }
                    }
                }
                
                if id.is_none() {
                    error!("ID: {:#?}", id);
                    error!("jsonrpc: {:#?}", jsonrpc);
                    error!("result: {:#?}", result);
                    panic!("No 'ID' in deserialized Response!");
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;

                let result = result.ok_or_else(|| de::Error::missing_field("result"))?;
                let mut de_res = ApiCall::from_id_and(id, |s| {
                    let string = format!(r#"{{  "{}":"{}"  }}"#, s, result);
                    debug!("{} = {}", "JSON String".red().bold(), &string.yellow().bold());
                    let res: std::result::Result<ResponseObject, JError> = serde_json::from_str(&string);
                    match res {
                        Ok(v) => v,
                        Err(e) => {
                            error!("{:#?}", e);
                            error!("{:#?}", std::error::Error::cause(&e));
                            error!("{:#?}", e.description());
                            panic!("{}: {}", "Could not deserialize eth call".magenta().bold().underline(), s.yellow().bold());
                        }
                    }
                });

               /*de_res = match de_res {
                    Ok(v) => v,
                    Err(e) => {
                        error!("{}", e);
                        error!("{}", e.cause());
                        panic!("Could not deserialize eth");
                    }
                }; */
                let jsonrpc = jsonrpc.ok_or_else(|| de::Error::missing_field("jsonrpc"))?;
                Ok(JsonBuilder {
                    jsonrpc,
                    id,
                    result: de_res,
                    method: None,
                    params: Vec::new(),
                })
            }
        }
        const FIELDS: &'static [&'static str] = &["id", "jsonrpc", "result", "method", "params"];
        deserializer.deserialize_struct("JsonBuilder", FIELDS, JsonBuilderVisitor)
    }
}

macro_rules! de_response {
    ($id: expr ) => ({
        ApiCall::from_id_and($id, |s| {  })
    })
}
/*
macro_rules! rpc_call {
    ($call:ident, $sel: ident) => ({
        match JsonBuilder::default().method(ApiCall::$call).build().map_err(|e| futures::future::err(e.into())) {
                Ok(j) => Box::new($sel.do_post(j)),
                Err(e) => Box::new(e)
            }
    })
}

*/

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;
    #[test]
    fn it_should_create_json() {
        env_logger::try_init();
        let test = json!({
           "id": 1,
           "jsonrpc": "2.0",
           "method": "eth_blockNumber",
           "params": [],
        }).to_string();

        let json = JsonBuilder::default().method(ApiCall::EthBlockNumber).build().expect("Bulding JSON failed");
        info!("{}:{:?}", "JSON OBJECT".cyan().bold(), json);
        assert_eq!(test, json);
    }
}
