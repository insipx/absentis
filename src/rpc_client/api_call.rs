use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;

macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Primitive)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$name, E>
                    where
                        E: ::serde::de::Error,
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}

// ETH id namespace == 00
// Net id namespace == 10
// Trace id namespace == 01
enum_number!(ApiCall {
    Nil                                     = 0,
    // Eth
    EthAccounts                             = 001,  // eth_accounts
    EthBlockNumber                          = 002,  // eth_blockNumber
    EthGetBlockByNumber                     = 003,  // eth_getBlockByNumber
    EthGasPrice                             = 004,  // eth_gasPrice
    EthGetBalance                           = 005,  // eth_getBalance
    EthGetBlockByHash                       = 006,  // eth_getBlockByHash
    EthGetTransactionByReceipt              = 007,  // eth_getTransactionByReceipt
    EthGetBlockTransactionCountByHash       = 008,  // eth_getBlockTransactionCountByHash
    EthGetBlockTransactionCountByNumber     = 009,  // eth_getBlockTransactionCountByNumber
    EthGetCode                              = 0010, // eth_getLogs
    EthGetLogs                              = 0011, // eth_getStorageAt
    EthGetStorageAt                         = 0012, // eth_getTransactionByBlockHashAndIndex
    EthGetTransactionByBlockHashAndIndex    = 0013, // eth_getTransctionByBlockNumberAndIndex
    EthGetTransactionByBlockNumberAndIndex  = 0014, 
    EthGetUncleByBlockNumberAndIndex        = 0015,
    EthGetUncleByBlockHashAndIndex          = 0016,
    EthGetUncleCountByBlockHash             = 0017,
    EthGetUncleCountByBlockNumber           = 0018,
    EthGetWork                              = 0019,
    EthHashrate                             = 0020,
    EthMining                               = 0021,
    EthProtocolVersion                      = 0022,
    EthSyncing                              = 0023,
    EthGetTransactionByHash                 = 0024,
    EthGetTransactionCount                  = 0025,

    // NET
    NetListening                            = 1001,
    NetPeerCount                            = 1002,
    NetVersion                              = 1003,

    // TRACE (Parity only)
    TraceCall                               = 0101,
    TraceRawTransaction                     = 0102,
    TraceReplayTransaction                  = 0103,
    TraceReplayBlockTransaction             = 0104,
    TraceBlock                              = 0105,
    TraceFilter                             = 0106,
    TraceGet                                = 0107,
    TraceTransaction                        = 0108,
});

impl std::fmt::Display for ApiCall {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Call: {}, representing: {}", self.to_str(), self.method_info())
    }

}

impl ApiCall {

    pub fn method_info(&self) -> String {
        match self {
            ApiCall::EthBlockNumber => "eth_blockNumber".to_string(),
            ApiCall::EthGetBlockByNumber => "eth_getBlockByNumber".to_string(),
            ApiCall::EthGasPrice => "eth_gasPrice".to_string(),
            ApiCall::EthGetBalance => "eth_getBalance".to_string(),
            _=> panic!("Api Call Does not exist")
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            ApiCall::EthBlockNumber => "EthBlockNumber".to_owned(),
            ApiCall::EthGetBlockByNumber => "EthGetBlockByNumber".to_owned(),
            ApiCall::EthGasPrice => "EthGasPrice".to_owned(),
            ApiCall::EthGetBalance => "EthGetBalance".to_owned(),
            _=> panic!("Api Call does not exist!")
        }
    }

    pub fn from_id_and<F,T>(id: usize, fun: F) -> T
        where
            F: FnOnce(ApiCall) -> T
    {
        match Self::from_usize(id).unwrap() { // TODO remove unwrap #p3
            c @ ApiCall::EthBlockNumber => fun(c),
            c @ ApiCall::EthGetBlockByNumber => fun(c),
            _ => panic!("Api Call does not exist!")
        }
        
    }
}

