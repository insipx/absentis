use serde_derive::{Deserialize};
use ethereum_types::{H256, Address};
use super::hex::Hex;

#[derive(Deserialize, Debug)]
pub struct Transaction {
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
