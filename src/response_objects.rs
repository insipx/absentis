use serde_json::*;
use serde_derive::*;
use ethereum_types::H256;

pub enum ResponseObjects {



}


#[derive(Serialize, Deserialize)]
struct BlockNumber {
  block_number: usize,
}


struct BlockByNumber {
  number: usize,
  difficulty: u64,
  extraData: String,
  gasLimit: u64,
  gasUsed: u64,
  hash: H256,
} 