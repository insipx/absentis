use log::*;
use serde_derive::*;
use serde_json::{self, from_str, from_slice, Error as JError, json, json_internal};
use serde::de::{self, Deserializer, Deserialize, Visitor, SeqAccess, MapAccess};
use serde_hex::{SerHexSeq,StrictPfx,CompactPfx};
use ethereum_types::*;
use colored::Colorize;
use http::Response;
use crate::json_builder::{JsonBuilder, JsonBuildError};
use crate::types::ApiCall;

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseObject {
    EthBlockNumber(Hex),
    EthGetBlockByNumber(Block),
    Nil,
}

impl ResponseObject {
    pub fn new(body: String) -> std::result::Result<Self, JsonBuildError> {
        debug!("{}: {:#?}", "JSON Response Result Object".cyan(), body.yellow());
        let json: JsonBuilder = serde_json::from_str(&body)?;
        Ok(json.get_result())
    }

    pub fn from_bytes(body: bytes::Bytes) -> std::result::Result<Self, JsonBuildError> {
        debug!("{}: {}", "JSON Response Result Object".cyan().bold(), std::str::from_utf8(&*body).unwrap().yellow().bold());
        debug!("In Function {} in file {}; line: {}", "`from_bytes`".bold().underline().bright_cyan(), file!().bold().underline(), line!().to_string().bold().bright_white().underline());
        let json: JsonBuilder = serde_json::from_slice(&body.to_vec())?;
        debug!("{}: {:?}", "JSON Response Object, Deserialized".cyan().bold(), json);
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

/*
impl<'de> Deserialize<'de> for StrOrMap {
    fn deserialize<D>(deserializer: D) -> std::result::Result<StrOrMap, D::Error> where D: Deserializer<'de> {

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {Strinter, Map}
        

        struct StrOrMapVisitor;

        impl<'de> Visitor<'de> for StrOrMapVisitor {
            type Value = StrOrMap;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct StrOrMap")
            }

            fn visit_map<V>(self, mut map: V) -> Result<StrOrMap, V::Error> 
            where
                V: MapAccess<'de>
            {
                let mut strinter = None;
                let mut map = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Strinter => panic!("Nope"),
                        Field::Map => {
                            
                        }
                    
                    }
                }
            
            }

            fn visit_str<E>(self, v: &str) -> Result<StrOrMap, E> 
            where
                E: de::Error
            {
                let mut strinter = v.to_owned();
                Ok(StrOrMap {
                    strinter,
                    map: None
                })
            }

            // implement visit_string if visit_str is not enough
        }
    }
}
*/

// #[derive(Deserialize, Serialize, Debug)]
// struct Hex(#[serde(with="SerHex::<StrictPfx>")] [u8; 32]);

//impl_serhex_bytearray!(Hex, 64);

#[derive(Deserialize, Serialize)]
pub struct Hex (
    #[serde(with ="SerHexSeq::<StrictPfx>")] 
    Vec<u8>
);


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
impl From<[u8; 64]> for Hex {
    fn from(arr: [u8; 64]) -> Hex {
        Hex(arr)
    }
}

impl std::convert::AsRef<[u8]> for Hex {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
*/
//#[derive(Serialize, Deserialize, Debug)]
// pub struct BlockNumber(Hex);

#[derive(Serialize, Deserialize, Debug)]
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
  transactions_objects: Option<Vec<Transaction>>,
  transactions_hashes: Option<Vec<H256>>
} 

#[derive(Serialize, Deserialize, Debug)]
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


/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_should_
}
*/
