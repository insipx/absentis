use log::*;
use serde_derive::*;
use serde_json::{self, from_str, from_slice, Error as JError, json, json_internal};
use serde::de::{self, Deserializer, Deserialize, Visitor, SeqAccess, MapAccess};
use hex::FromHex;
use ethereum_types::*;
use colored::Colorize;
use failure::{Error as FError, Fail};
use crate::utils::*;
use crate::json_builder::{JsonBuilder, JsonBuildError};
use crate::types::ApiCall;

#[derive(Debug, Deserialize)]
pub enum ResponseObject {
    EthBlockNumber(Hex),
    EthGetBlockByNumber(Block),
    Nil,
}

#[derive(Fail, Debug)]
pub struct TypeMismatchError {
    invalid_type: String
}

impl TypeMismatchError {
    fn new(err: String) -> Self {
        TypeMismatchError {
            invalid_type: err
        }
    }
}

impl std::fmt::Display for TypeMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.invalid_type)
    }
}

#[derive(Fail, Debug)]
pub enum ResponseBuildError {
    #[fail(display = "Error building JSON Object from 'Result'")]
    SerializationError(#[fail(cause)] serde_json::error::Error),
    #[fail(display = "Hyper Error while building Json Response Object")]
    HyperError(#[fail(cause)] hyper::error::Error),
    #[fail(display = "Mismatched types during build")]
    MismatchedTypes(TypeMismatchError)
}

impl From<serde_json::error::Error> for ResponseBuildError {
    fn from(err: serde_json::error::Error) -> Self {
        ResponseBuildError::SerializationError(err)
    }
}

// expects a string and value 
macro_rules! mismatched_types {
    ($expected_type: expr, $recvd_type: ident) => ({
        let string = format!("Expected type `{}`, got `{}` in {}", $expected_type, $recvd_type, err_loc!());
        Err(ResponseBuildError::MismatchedTypes(TypeMismatchError::new(string)))
    })
}


impl ResponseObject {
    pub fn new(body: String) -> std::result::Result<Self, ResponseBuildError> {
        debug!("{}: {:#?}", "JSON Response Result Object".cyan(), body.yellow());
        let json: JsonBuilder = serde_json::from_str(&body)?;
        Ok(json.get_result())
    }
    
    // parses a serde_json::Value into a ResponseObject
    // Value must be a Value::String or Value::Object
    pub fn from_serde_value(mut val: serde_json::Value, id: usize) -> Result<Self, ResponseBuildError> {
        match ApiCall::from_id(id) {
            ApiCall::EthBlockNumber => {
                if !val.is_string() {   
                    mismatched_types!("String", val)
                } else {
                    let hex = serde_json::from_str(&val.take().to_string());
                    Ok(ResponseObject::EthBlockNumber(verb_err!(hex)))
                }
            },
            ApiCall::EthGetBlockByNumber => {
                if !val.is_object() {
                    mismatched_types!("Map", val)
                } else {
                    debug!("Map String: {}", val.to_string().yellow().bold());
                    let block = serde_json::from_str(&val.take().to_string());
                    Ok(ResponseObject::EthGetBlockByNumber(verb_err!(block)))
                }
            }
        }
    }
    
    pub fn from_bytes(body: bytes::Bytes) -> std::result::Result<Self, JsonBuildError> {
        // debug!("{}: {}", "JSON Response Result Object".cyan().bold(), std::str::from_utf8(&*body).unwrap().yellow().bold());
        // debug!("In Function {} in file {}; line: {}", "`from_bytes`".bold().underline().bright_cyan(), file!().bold().underline(), line!().to_string().bold().bright_white().underline());
        let json: JsonBuilder = serde_json::from_slice(&body.to_vec())?;
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

#[derive(Deserialize, Debug)]
pub struct Block {
  number: Hex,
  hash: H256,
  parentHash: H256,
  nonce: H64,
  sha3Uncles: H256,
  logsBloom: Bloom,
  transactionsRoot: H256,
  stateRoot: H256,
  receiptsRoot: H256,
  miner: H160,
  difficulty: Hex,
  totalDifficulty: Hex,
  extraData: Hex,
  size: Hex,
  gasLimit: Hex,
  gasUsed:  Hex,
  timestamp: Hex,
  #[serde(rename = "transactions")]
  transactions_objects: Option<Vec<Transaction>>,
  #[serde(rename = "transactions")]
  transactions_hashes: Option<Vec<H256>>
} 

#[derive(Deserialize, Debug)]
struct Transaction {
  hash: H256,
  nonce: Hex,
  blockHash: H256,
  blockNumber: Hex,
  transactionIndex: Hex,
  from: Address,
  to: Address,
  value: Hex, 
  gasPrice: Hex,
  gas: Hex,
  input: Hex,
}

pub struct Hex (Vec<u8>);

impl<'de> Deserialize<'de> for Hex {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {

        struct HexVisitor;

        impl<'de> Visitor<'de> for HexVisitor {
            type Value = Hex;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Hex")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: de::Error {

                let rmv_pfx = | x: &str | return x.char_indices().skip_while(|(i, x)| *i < 2).map(|(_, x)| x).collect::<String>();
                let get_hex = | x: &str | return Vec::from_hex(x).map_err(|e| de::Error::custom(e));
               

                if v.starts_with("0x") && v.len() % 2 == 0 {
                    Ok( Hex(get_hex(&rmv_pfx(v))?) )
                
                } else if v.starts_with("0x") && v.len() % 2 != 0 {
                    let no_pfx = rmv_pfx(v);
                    let make_even = format!("{}{}", "0", no_pfx);
                    Ok( Hex(get_hex(&make_even)?) )
                
                } else if v.len() % 2 != 0 {
                    let make_even = format!("{}{}", "0", v);
                    Ok( Hex(get_hex(&make_even)?) )
                
                } else {
                    Ok( Hex(get_hex(v)?) )
                }
            }
        }
        deserializer.deserialize_str(HexVisitor)
    }
}

impl std::fmt::Debug for Hex {
    /*fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{}", self.0.iter().map(|x| format!("{:x}", x)).collect::<String>())
    }*/
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let hex_str: String = self.0.iter().map(|b|  format!("{:x}", b)).collect();
        write!(f, "0x{}", hex_str)
    }
}



/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_should_
}
*/
