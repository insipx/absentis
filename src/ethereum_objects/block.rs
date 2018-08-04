use serde_derive::{Deserialize};
use ethereum_types::{H256, Bloom, H160, H64};
use super::transaction::Transaction;
use super::hex::Hex;


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
