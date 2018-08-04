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
    #[fail(display = "Error building JsonBuild JSON Object")]
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
        debug!("{}: {}", "VAL".cyan().underline().bold(), val.to_str().cyan().bold());
        let (id, method) = val.method_info();
        debug!("ID: {}, METHOD: {}", id.to_string().underline().blue(), method.underline().blue());
        new.id = id;
        new.method = Some(method);
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

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                      A Note on the current Deserialization of JsonBuilder
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Serde-Json has a `Preserve Order` feature that is enabled on this crate: https://github.com/serde-rs/json/issues/54 
// this means that we can be *fairly* certain that ID will be parsed before `result`. However, this is not foolproof,
// since the order of the mapping is left up to the JSONRPC Server. In a Seq Response (array), the
// order may be totally different as well. This essentially means that if any server in the future decides to change there API all willy-nilly
// Lots of errors may be produced here.
// Another option is to create an intermediate struct representation of JsonBuilder, 
// that 'flattens' the enum representation into one struct representation. 
// IE:
// ```
// {
//  struct MyHappyStruct {
//      num: usize,
//      a_str: String,
//      anEnum: Foo
//  }
//
//  enum Foo {
//      A(Data),
//      B(OtherData),
//      C(MoreData)
//  }
//
//  in `deserialize()`
//  struct Mapping {
//      num: usize,
//      a_str: String,
//      #[serde(rename = A)]
//      a: Option<Data>
//      #[serde(rename = B)]
//      b: Option<OtherData>
//      #[serde(rename = C)]
//      c: Option<MoreData>
//  }
// }
// ```
// This came from a SO thread here: 
// https://stackoverflow.com/questions/45059538/how-to-deserialize-into-a-enum-variant-based-on-a-key-name
// This way is significantly more tedious, however. It can be left up to macros, but I will leave that for a
// future release TODO: unwrap enum into intermediate struct representation of JsonBuilder #p3
// Another option is to use 'Struct or String' but adapt it to an enum, and map, ie 'Map or
// String'. This requires the use of `deserializer.any()`. The implementation below follows a
// combination of both these suggestions. (whatever worked when I came up with it)
//
// It relies on ID being deserialized first. Once ID is deserialized, the correct ResponseObject::
// enum variant can be chosen (it is a 'MUST' of JSONRPC spec to return the same 'id' that was sent
// by a client). 
// EDIT: Preserve ord makes no diff
/////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////// - insidious //////////////////////////////////////////////

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
                let mut result: Option<ResponseObject> = None;

                /*** SEE: Deserialization Note! ***/
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
                            let id = id.ok_or_else(|| de::Error::custom("Id is none! Serde did not preserve order, or \
                                                                 JSON from RPC did not respond with `id` before `result`"));
                            let map_or_str: serde_json::Value = map.next_value()?;
                            // let res = ResponseObject::from_value(value, id)
                            result = Some(ResponseObject::from_serde_value(map_or_str, id?).map_err(|e| de::Error::custom(e))?)
                        },
                        Field::Method => { // skip
                            /* return Err(de::Error::unknown_field("Don't deserialize 'Method'", map.next_value()?));*/
                        },
                        Field::Params => { // skip
                            /* return Err(de::Error::unknown_field("Don't deserialize 'Params'", map.next_value()?)); */
                        }
                    }
                }
                
                let id = id.expect("For execution to get to this point, id must have been used succesfully during the `result` match; qed");
                let result = result.ok_or_else(|| de::Error::missing_field("result"))?;
                debug!("{}: {}","ID".red().bold(), id);

                let jsonrpc = jsonrpc.ok_or_else(|| de::Error::missing_field("jsonrpc"))?;
                Ok(JsonBuilder {
                    jsonrpc,
                    id,
                    result,
                    method: None,
                    params: Vec::new(),
                })
            }

            /* fn visit_seq */ // this function would be used if any of the Ethereum JSONRPC's
            // returned responses as positional arrays, not Objects. So far none do, so there is no
            // need to implement this as of yet.
            // TODO: implement `visit_seq` for JsonBuilder #p3
        }
        const FIELDS: &'static [&'static str] = &["id", "jsonrpc", "result", "method", "params"];
        deserializer.deserialize_struct("JsonBuilder", FIELDS, JsonBuilderVisitor)
    }
}



pub trait ToMap {
    fn to_map(&self) -> String;
}


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
