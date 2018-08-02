use log::*;
use serde_json::*;
use serde_derive::*;
use ethereum_types::*;
use colored::Colorize;
use http::Response;
use serde::de;
use failure::Error;
use crate::json_builder::{JsonBuilder, JsonBuildError};
use crate::types::ApiCall;

#[derive(Debug)]
pub enum ResponseObject {
  EthBlockNumber(u64),
  EthGetBlockByNumber(Block),
}

impl ResponseObject {
    pub fn new(body: String) -> std::result::Result<Self, JsonBuildError> {
        debug!("{}: {:#?}", "JSON Response Result Object".cyan(), json.get_result().yellow());
        let json: JsonBuilder = serde_json::from_str(&body)?;
        match ApiCall::from_id(json.get_id()) {
            EthBlockNumber => Ok(ResponseObject::EthBlockNumber(serde_json::from_str(&json.get_result())?)),
            EthGetBlockByNumber => Ok(ResponseObject::EthGetBlockByNumber(serde_json::from_str(&json.get_result())?))
        }
    }

    pub fn from_bytes(mut body: bytes::Bytes) -> std::result::Result<Self, JsonBuildError> {
        debug!("{}: {:#?}", "JSON Response Result Object".cyan(), json.get_result().yellow());
        let json: JsonBuilder = serde_json::from_slice(&body.to_vec())?;
        match ApiCall::from_id(json.get_id()) {
            EthBlockNumber => Ok(ResponseObject::EthBlockNumber(serde_json::from_str(&json.get_result())?)),
            EthGetBlockByNumber => Ok(ResponseObject::EthGetBlockByNumber(serde_json::from_str(&json.get_result())?))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BlockNumber {
  block_number: usize,
}

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
