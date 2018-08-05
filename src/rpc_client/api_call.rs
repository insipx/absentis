pub enum ApiCall {
    EthBlockNumber,
    EthGasPrice,
    EthGetBalance,
    EthGetBlockByHash,
    EthGetBlockByNumber,
    EthGetTransactionByReceipt,
    EthGetBlockTransactionCountByHash,
    EthGetBlockTransactionCountByNumber,
    EthGetCode,
    EthGetLogs,
    EthGetStorageAt,
    EthGetTransactionByBlockHashAndIndex,
    EthGetTransactionByBlockNumberAndIndex,
    EthGetUncleByBlockNumberAndIndex,
    EthGetUncleByBlockHashAndIndex,
    EthGetUncleCountByBlockHash,
    EthGetUncleCountByBlockNumber,
    EthGetWork,
    EthHashrate,
    EthMining,
    EthProtocolVersion,
    EthSyncing,
    EthGetTransactionByHash,
    EthGetTransactionCount,

    // Net,
    NetListening,
    NetPeerCount,
    NetVersion,

    // (Parity Only),
    TraceCall,
    TraceRawTransaction,
    TraceReplayTransaction,
    TraceReplayBlockTransaction,
    TraceBlock,
    TraceFilter,
    TraceGet,
    TraceTransaction,
}

impl std::fmt::Display for ApiCall {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Call: {}, representing: {}", self.to_str(), self.method())
    }

}

impl ApiCall {

    pub fn method(&self) -> String {
        match self {
            ApiCall::EthBlockNumber                         => "eth_blockNumber".to_string(),
            ApiCall::EthGasPrice                            => "eth_gasPrice".to_string(),
            ApiCall::EthGetBalance                          => "eth_getBalance".to_string(),
            ApiCall::EthGetBlockByHash                      => "eth_getBlockByHash".to_string(),
            ApiCall::EthGetBlockByNumber                    => "eth_getBlockByNumber".to_string(),
            ApiCall::EthGetTransactionByReceipt             => "eth_getBlockTransactionCountByNumber".to_string(),
            ApiCall::EthGetBlockTransactionCountByHash      => "eth_getBlockTransactionCountByHash".to_string(),
            ApiCall::EthGetBlockTransactionCountByNumber    => "eth_getBlockTransactionCountByNumber".to_string(),
            ApiCall::EthGetCode                             => "eth_getCode".to_string(),
            ApiCall::EthGetLogs                             => "eth_getLogs".to_string(),
            ApiCall::EthGetStorageAt                        => "eth_getStorageAt".to_string(),
            ApiCall::EthGetTransactionByBlockHashAndIndex   => "eth_getTransactionByBlockHashAndIndex".to_string(),
            ApiCall::EthGetTransactionByBlockNumberAndIndex => "eth_getTransactionByBlockNumberAndIndex".to_string(),
            ApiCall::EthGetUncleByBlockNumberAndIndex       => "eth_getUncleByBlockNumberAndIndex".to_string(),
            ApiCall::EthGetUncleByBlockHashAndIndex         => "eth_getUncleByBlockHashAndIndex".to_string(),
            ApiCall::EthGetUncleCountByBlockHash            => "eth_getUncleCountByBlockHash".to_string(),
            ApiCall::EthGetUncleCountByBlockNumber          => "eth_getUncleCountByBlockNumber".to_string(),
            ApiCall::EthGetWork                             => "eth_getWork".to_string(),
            ApiCall::EthHashrate                            => "eth_gashrate".to_string(),
            ApiCall::EthMining                              => "eth_mining".to_string(),
            ApiCall::EthProtocolVersion                     => "eth_protocolVersion".to_string(),
            ApiCall::EthSyncing                             => "eth_syncing".to_string(),
            ApiCall::EthGetTransactionByHash                => "eth_getTransactionByHash".to_string(),
            ApiCall::EthGetTransactionCount                 => "eth_getTransactionCount".to_string(),

             // Net
            ApiCall::NetListening                           => "net_listening".to_string(),
            ApiCall::NetPeerCount                           => "net_peerCount".to_string(),
            ApiCall::NetVersion                             => "net_version".to_string(),

            // Trace (Parity Only)
            ApiCall::TraceCall                              => "trace_call".to_string(),
            ApiCall::TraceRawTransaction                    => "trace_rawTransaction".to_string(),
            ApiCall::TraceReplayTransaction                 => "trace_replayTransaction".to_string(),
            ApiCall::TraceReplayBlockTransaction            => "trace_replayBlockTransaction".to_string(),
            ApiCall::TraceBlock                             => "trace_block".to_string(),
            ApiCall::TraceFilter                            => "trace_filter".to_string(),
            ApiCall::TraceGet                               => "trace_get".to_string(),
            ApiCall::TraceTransaction                       => "trace_transaction".to_string(),
            _=> panic!("Api Call Does not exist")
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            ApiCall::EthBlockNumber                          => "EthBlockNumber"     .to_string(),
            ApiCall::EthGasPrice                             => "EthGasPrice"        .to_string(),
            ApiCall::EthGetBalance                           => "EthGetBalance"      .to_string(),
            ApiCall::EthGetBlockByHash                       => "EthGetBlockByHash"  .to_string(),
            ApiCall::EthGetBlockByNumber                     => "EthGetBlockByNumber".to_string(),
            ApiCall::EthGetTransactionByReceipt              => "EthGetTransactionByReceipt".to_string(),
            ApiCall::EthGetBlockTransactionCountByHash       => "EthGetBlockTransactionCountByHash".to_string(),
            ApiCall::EthGetBlockTransactionCountByNumber     => "EthGetBlockTransactionCountByNumber".to_string(),
            ApiCall::EthGetCode                              => "EthGetCode".to_string(),
            ApiCall::EthGetLogs                              => "EthGetLogs".to_string(),
            ApiCall::EthGetStorageAt                         => "EthGetStorageAt".to_string(),
            ApiCall::EthGetTransactionByBlockHashAndIndex    => "EthGetTransactionByBlockHashAndIndex".to_string(),
            ApiCall::EthGetTransactionByBlockNumberAndIndex  => "EthGetTransactionByBlockNumberAndIndex".to_string(),
            ApiCall::EthGetUncleByBlockNumberAndIndex        => "EthGetUncleByBlockNumberAndIndex".to_string(),
            ApiCall::EthGetUncleByBlockHashAndIndex          => "EthGetUncleByBlockHashAndIndex".to_string(),
            ApiCall::EthGetUncleCountByBlockHash             => "EthGetUncleCountByBlockHash".to_string(),
            ApiCall::EthGetUncleCountByBlockNumber           => "EthGetUncleCountByBlockNumber".to_string(),
            ApiCall::EthGetWork                              => "EthGetWork".to_string(),
            ApiCall::EthHashrate                             => "EthHashrate".to_string(),
            ApiCall::EthMining                               => "EthMining".to_string(),
            ApiCall::EthProtocolVersion                      => "EthProtocolVersion".to_string(),
            ApiCall::EthSyncing                              => "EthSyncing".to_string(),
            ApiCall::EthGetTransactionByHash                 => "EthGetTransactionByHash".to_string(),
            ApiCall::EthGetTransactionCount                  => "EthGetTransactionCount".to_string(),

             // Net
            ApiCall::NetListening                            => "NetListening".to_string(),
            ApiCall::NetPeerCount                            => "NetPeerCount".to_string(),
            ApiCall::NetVersion                              => "NetVersion".to_string(),

            // Trace (Parity Only)
            ApiCall::TraceCall                               => "TraceCall".to_string(),
            ApiCall::TraceRawTransaction                     => "TraceRawTransaction".to_string(),
            ApiCall::TraceReplayTransaction                  => "TraceReplayTransaction".to_string(),
            ApiCall::TraceReplayBlockTransaction             => "TraceReplayBlockTransaction".to_string(),
            ApiCall::TraceBlock                              => "TraceBlock".to_string(),
            ApiCall::TraceFilter                             => "TraceFilter".to_string(),
            ApiCall::TraceGet                                => "TraceGet".to_string(),
            ApiCall::TraceTransaction                        => "TraceTransaction".to_string(),
            _ => panic!("Api Call Does not exist!")
        }
    }
}

