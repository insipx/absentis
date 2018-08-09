use super::ResponseObject;
use crate::ethereum_objects::EthObjType;
use super::err::ResponseBuildError;

#[macro_export]
macro_rules! result {
    ($result: expr, $err: ident) => ({
        if !$err.is_some() {
            return Ok($result)
        } else {
            let err_info = $err.unwrap().info();
            Err(ResponseBuildError::RPCError(err_info.0, err_info.1))
        }
    });
}

// returns either Err(RPCError) or Ok(EthObjType::)
#[macro_export]
macro_rules! match_response {
    ($resp: ident) => ({
        match $resp {
            ResponseObject::EthBlockNumber{result, err}                         => result!(EthObjType::Hex(result), err),
            ResponseObject::EthGetBlockByNumber{result, err}                    => result!(EthObjType::Block(result), err),
            ResponseObject::EthGasPrice{result, err}                            => result!(EthObjType::Hex(result), err),
            ResponseObject::EthGetBalance{result, err}                          => result!(EthObjType::Hex(result), err),
            ResponseObject::EthGetBlockByHash{result, err}                      => result!(EthObjType::Block(result), err),
            /*
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
            ResponseObject::TraceTransaction(result, err)                       => fun(&err, &result), */
        }
    })
}

