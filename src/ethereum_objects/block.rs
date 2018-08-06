use serde_derive::{Deserialize};
use ethereum_types::{H256, Bloom, H160, H64};
use super::transaction::Transaction;
use super::hex::Hex;


#[derive(Deserialize, Debug)]
pub struct Block {
    number: Hex,
    hash: H256,
    #[serde(rename="parentHash")]
    parent_hash: H256,
    nonce: H64,
    #[serde(rename="sha3Uncles")]
    sha3_uncles: H256,
    #[serde(rename="logsBloom")]
    logs_bloom: Bloom,
    #[serde(rename ="transactionsRoot")]
    transactions_root: H256,
    #[serde(rename="stateRoot")]
    state_root: H256,
    #[serde(rename="receiptsRoot")]
    receipts_root: H256,
    miner: H160,
    difficulty: Hex,
    #[serde(rename="totalDifficulty")]
    total_difficulty: Hex,
    #[serde(rename="extraData")]
    extra_data: Hex,
    size: Hex,
    #[serde(rename="gasLimit")]
    gas_limit: Hex,
    #[serde(rename = "gasUsed")]
    gas_used:  Hex,
    timestamp: Hex,
    transactions: Vec<Option<Transaction>>, // tx is either `Transaction` or `H256`
}


