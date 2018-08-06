use serde_derive::{Deserialize};
use ethereum_types::{H256, Address};
use super::hex::Hex;

#[derive(Deserialize, Debug)]
pub struct Transaction {
  hash: H256,
  nonce: Hex,
  #[serde(rename="blockHash")]
  block_hash: Option<H256>,
  #[serde(rename="blockNumber")]
  block_number: Option<Hex>,
  #[serde(rename="transactionIndex")]
  transaction_index: Option<Hex>,
  from: Address,
  to: Option<Address>,
  value: Hex, 
  #[serde(rename="gasPrice")]
  gas_price: Hex,
  gas: Hex,
  input: Hex,
}
