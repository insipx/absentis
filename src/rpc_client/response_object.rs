//! Object holding possible responses from JSON-RPC's (Parity, Infura, Geth, etc)
use log::{debug, log, info};
use serde_derive::{Deserialize};
use serde::de::{Deserialize, Deserializer, Visitor, EnumAccess};
use colored::Colorize;
use num_traits::FromPrimitive;
use failure::{Fail};

use crate::ethereum_objects::{Hex, Block, EthObjType};
use super::err::{ResponseBuildError, TypeMismatchError};
use super::api_call::ApiCall;

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
    (EthBlockNumber, Hex),(EthGasPrice, Hex),(EthGetBalance, Hex),(EthGetBlockByHash,Block),
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
    /*
    pub fn new(body: String) -> std::result::Result<Self, ResponseError> {
        debug!("{}: {:#?}", "JSON Response Result Object".cyan(), body.yellow());
        serde_json::from_str(&body)?
    }
    */
    /*
    fn is_error(&self) -> bool {
        self.error.is_some()
    } 
    */
    
    pub fn from_bytes(body: bytes::Bytes) -> std::result::Result<Self, ResponseError> {
        let json: ResponseObject = serde_json::from_slice(&body.to_vec())?;
        /*
        if json.is_error() {
            // info!("JsonResponse: {:?}", json);
            // let err_info = json.err_info().unwrap();
            // return Err(ResponseError::RPCError(err_info.0, err_info.1))
            panic!("Uncomment the code");
        }
        */
        Ok(json)
    }
}




#[derive(Deserialize, Debug)]
pub struct JsonRpcError  {
    code: i64,
    message: String,
    data: Option<serde_json::Value>
}

impl JsonRpcError {
    pub fn info(&self) -> (String, i64) {
        (self.message.clone(), self.code)
    }
}



#[derive(Fail, Debug)]
pub enum ResponseError {
    #[fail(display = "Error deserializing Response Object: {}", _0)]
    DeserializationError(#[cause] serde_json::error::Error),
    #[fail(display = "Hyper Error receiving Response object, {}", _0)]
    HyperError(#[cause] hyper::error::Error),
    #[fail(display = "The Ethereum JsonRPC returned an error: {}, code: {}", _0, _1)]
    RPCError(String, i64)
}


impl From<hyper::error::Error> for ResponseError {
    fn from(err: hyper::error::Error) -> ResponseError {
        ResponseError::HyperError(err)
    }
}

impl From<serde_json::error::Error> for ResponseError {
    fn from(err: serde_json::error::Error) -> ResponseError {
        ResponseError::DeserializationError(err)
    }
}


/*
pub fn inner_enum<F, T>(resp: ResponseObject, fun: F) -> T
    where F: Fn(Option<JsonRpcError>, EthObjType) -> T
{
    match resp {
        // Eth
        // ResponseObject::EthAccounts(result, err)                            => func(&err, &result),
        ResponseObject::EthBlockNumber{result, err}                         => fun(err, EthObjType::Hex(result)),
        ResponseObject::EthGetBlockByNumber{result, err}                    => fun(err, EthObjType::Block(result)),
        ResponseObject::EthGasPrice{result, err}                            => fun(err, EthObjType::Hex(result)),
        ResponseObject::EthGetBalance{result, err}                          => fun(err, EthObjType::Hex(result)),
        /*
        ResponseObject::EthGetBlockByHash(result, err)                      => fun(&err, &result),
        ResponseObject::EthGetTransactionByReceipt(result, err)             => fun(&err, &result),
        ResponseObject::EthGetBlockTransactionCountByHash(result, err)      => fun(&err, &result),
        ResponseObject::EthGetBlockTransactionCountByNumber(result, err)    => fun(&err, &result),
        ResponseObject::EthGetCode(result, err)                             => fun(&err, &result),
        ResponseObject::EthGetLogs(result, err)                             => fun(&err, &result),
        ResponseObject::EthGetStorageAt(result, err)                        => fun(&err, &result),
        ResponseObject::EthGetTransactionByBlockHashAndIndex(result, err)   => fun(&err, &result),
        ResponseObject::EthGetTransactionByBlockNumberAndIndex(result, err) => fun(&err, &result),
        ResponseObject::EthGetUncleByBlockNumberAndIndex(result, err)       => fun(&err, &result),
        ResponseObject::EthGetUncleByBlockHashAndIndex(result, err)         => fun(&err, &result),
        ResponseObject::EthGetUncleCountByBlockHash(result, err)            => fun(&err, &result),
        ResponseObject::EthGetUncleCountByBlockNumber(result, err)          => fun(&err, &result),
        ResponseObject::EthGetWork(result, err)                             => fun(&err, &result),
        ResponseObject::EthHashrate(result, err)                            => fun(&err, &result),
        ResponseObject::EthMining(result, err)                              => fun(&err, &result),
        ResponseObject::EthProtocolVersion(result, err)                     => fun(&err, &result),
        ResponseObject::EthSyncing(result, err)                             => fun(&err, &result),
        ResponseObject::EthGetTransactionByHash(result, err)                => fun(&err, &result),
        ResponseObject::EthGetTransactionCount(result, err)                 => fun(&err, &result),

        // Net
        ResponseObject::NetListening(result, err)                           => fun(&err, &result),
        ResponseObject::NetPeerCount(result, err)                           => fun(&err, &result),
        ResponseObject::NetVersion(result, err)                             => fun(&err, &result),

        // TRACE (Parity only)
        ResponseObject::TraceCall(result, err)                              => fun(&err, &result),
        ResponseObject::TraceRawTransaction(result, err)                    => fun(&err, &result),
        ResponseObject::TraceReplayTransaction(result, err)                 => fun(&err, &result),
        ResponseObject::TraceReplayBlockTransaction(result, err)            => fun(&err, &result),
        ResponseObject::TraceBlock(result, err)                             => fun(&err, &result),
        ResponseObject::TraceFilter(result, err)                            => fun(&err, &result),
        ResponseObject::TraceGet(result, err)                               => fun(&err, &result),
        ResponseObject::TraceTransaction(result, err)                       => fun(&err, &result),
        _                                                                   => panic!("Api call does not exist");
        */
    }
}
*/
/*
impl<'de> Deserialize<'de> for ResponseObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> 
        where
            D: Deserializer<'de>,
    {
        let name: &'static str = "ResponseObject";
        let variants: &'static [&'static str] = &[
        "EthBlockNumber","EthGetBlockByNumber","EthGasPrice","EthGetBalance",
        ];
        deserializer.deserialize_enum(name, variants, ResponseObjectVisitor)
    }
}
*/
/*
use std::marker::PhantomData;

struct ResponseObjectVisitor<E: EthObj> { 
    _marker: PhantomData<E>
}

impl<'de, E> Visitor<'de> for ResponseObjectVisitor<E> where E: EthObj {
    type Value = ResponseObject<E>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("ResponseObject enum")
    }
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error> where A: EnumAccess<'de> {
        /*
        match data.variant().unwrap() {
            ResponseObject::EthBlockNumber{result, err} => println!("It's EthBlockNumber!"),
            ResponseObject::EthGetBlockByNumber{result, err} => println!("It's EthGetBlockByNumber!"),
            ResponseObject::EthGasPrice{result, err} => println!("It's eth gas price!"),
            _ => println!("Its some other bs")
        };
        */
        let fields = data.variant().unwrap();
        let hex: Hex = serde_json::from_str("0xff").unwrap();
        Ok(ResponseObject::EthGasPrice{result: hex, err: None})
    }
}
*/



