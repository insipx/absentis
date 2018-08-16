use serde_derive::*;
use web3::types::{H256, Bytes, Address};
use ::super::types::ETHERSCAN_URL;

// 60,137,282,256
#[derive(Deserialize)]
pub struct EtherScanTx {
    #[serde(rename = "blockNumber")]
    block_number: usize,
    #[serde(rename = "timeStamp")]
    time_stamp: u64,
    hash: H256,
    nonce: usize,
    #[serde(rename = "blockHash")]
    block_hash: H256,
    #[serde(rename = "transactionIndex")]
    transaction_index: usize,
    from: Address,
    to: Address,
    value: usize,
    gas: usize,
    #[serde(rename = "gasPrice")]
    gas_price: u64,
    #[serde(rename = "isError")]
    is_error: bool,
    txreceipt_status: String, // no rename needed
    input: Bytes,
    #[serde(rename = "contractAddress")]
    contract_address: String,
    #[serde(rename = "cumulative_gas_used")]
    cumulative_gas_used: usize,
    #[serde(rename = "gasUsed")]
    gas_used: usize,
    confirmations: usize,
}

// TODO: #p1 these should be functions, but i'm lazy
#[macro_export]
macro_rules! eth_txlist {
    ($addr:expr, $from:expr, $to:expr) => ({
        format!("{}?module=account&action=txlist&address={}&startblock={}&endblock={}&sort=asc&apikey=YourApiKeyToken",
                ETHERSCAN_URL,
                $addr,
                $from,
                $to
        );
    });
    ($addr:expr, $from:expr, $to:expr, $sort:expr) => ({
        format!("{}?module=account&action=txlist&address={}&startblock={}&endblock={}&sort={}&apikey=YourApiKeyToken",
                ETHERSCAN_URL,
                $addr,
                $from,
                $to,
                $sort
        );
    });
}



