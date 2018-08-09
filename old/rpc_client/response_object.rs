//! Object holding possible responses from JSON-RPC's (Parity, Infura, Geth, etc)
use serde_derive::{Deserialize};
use crate::ethereum_objects::{Hex, Block};
use super::err::{JsonRpcError, ResponseBuildError};

macro_rules! response_obj {
    ($name:ident { $(($variant:ident, $res: ident),)* }) => {
        #[derive(Debug, Deserialize)]
        #[serde(tag = "id")]
        pub enum $name {
            $($variant {result: $res, #[serde(rename = "error")] err: Option<JsonRpcError>}, )*
        }
    }
}

response_obj![ResponseObject {
    (EthBlockNumber, Hex),(EthGasPrice, Hex),(EthGetBalance, Hex),(EthGetBlockByHash, Block),
    (EthGetBlockByNumber, Block),
   
    /*
    (EthGetTransactionByReceipt,            Transaction),
    (EthGetBlockTransactionCountByHash,     Hex),
    (EthGetBlockTransactionCountByNumber,   Hex),
    (EthGetCode,                            Hex),
    (EthGetLogs,                            Hex),
    (EthGetStorageAt,                       Hex),
    (EthGetTransactionByBlockHashAndIndex,  Transaction),*/
}];


impl ResponseObject {
    pub fn from_bytes(body: bytes::Bytes) -> std::result::Result<Self, ResponseBuildError> {
        let json: ResponseObject = serde_json::from_slice(&body.to_vec())?;
        Ok(json)
    }
}

