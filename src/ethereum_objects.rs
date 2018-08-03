use log::*;
use serde_json::*;
use serde_derive::*;
use serde::de::Deserializer;
use serde_hex::{SerHexSeq,StrictPfx,CompactPfx};
use ethereum_types::*;
use colored::Colorize;
use http::Response;
use serde::de;
use failure::Error;
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
        /*
        match ApiCall::from_id(json.get_id()) {
            EthBlockNumber => Ok(ResponseObject::EthBlockNumber(serde_json::from_str(&json.get_result())?)),
            EthGetBlockByNumber => Ok(ResponseObject::EthGetBlockByNumber(serde_json::from_str(&json.get_result())?))
        }
        */
    }

    pub fn from_bytes(mut body: bytes::Bytes) -> std::result::Result<Self, JsonBuildError> {
        debug!("{}: {}", "JSON Response Result Object".cyan().bold(), std::str::from_utf8(&*body).unwrap().yellow().bold());
        let json: JsonBuilder = serde_json::from_slice(&body.to_vec())?;
        debug!("{}: {:?}", "JSON Response Object, Deserialized".cyan().bold(), json);
        // debug!("{}: {}", "JSON RESULT Object, Serialized".cyan().bold(), &json.get_result().yellow().bold());
        debug!("{}", r#"0x5cab"#);
        Ok(json.get_result())

        /*
        match ApiCall::from_id(json.get_id()) {
            EthBlockNumber => Ok(ResponseObject::EthBlockNumber(serde_json::from_str(&json.get_result())?)),
            EthGetBlockByNumber => Ok(ResponseObject::EthGetBlockByNumber(serde_json::from_str(&json.get_result())?))
        }
        */
    }
}


// #[derive(Deserialize, Serialize, Debug)]
// struct Hex(#[serde(with="SerHex::<StrictPfx>")] [u8; 32]);

//impl_serhex_bytearray!(Hex, 64);

#[derive(Deserialize, Serialize, Debug)]
pub struct Hex (
    #[serde(with ="SerHexSeq::<StrictPfx>")] 
    Vec<u8>
);


/*
impl std::fmt::Debug for Hex {
    /*fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{}", self.0.iter().map(|x| format!("{:x}", x)).collect::<String>())
    }*/
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
*/
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
  number: usize,
  hash: H256,
  parentHash: H256,
  nonce: H64,
  sha3Uncles: H256,
  logsBloom: Bloom,
  transactionsRoot: H256,
  stateRoot: H256,
  receiptsRoot: H256,
  miner: H160,
  difficulty: u64,
  totalDifficulty: u64,
  extraData: String,
  size: u64,
  gasLimit: u64,
  gasUsed:  u64,
  timestamp: String,
  transactions_objects: Option<Vec<Transaction>>,
  transactions_hashes: Option<Vec<H256>>
} 

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
  hash: H256,
  nonce: usize,
  blockHash: H256,
  blockNumber: usize,
  transactionIndex: usize,
  from: Address,
  to: Address,
  value: u64, 
  gasPrice: usize,
  gas: usize,
  input: String,
}


/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_should_
}
*/
