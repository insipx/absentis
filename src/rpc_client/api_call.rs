use serde_derive::{Serialize, Deserialize};
use enum_primitive_derive::Primitive;
use num_traits::{FromPrimitive, ToPrimitive};


#[derive(Serialize, Deserialize, Primitive, Debug)]
pub enum ApiCall {
    Nil = 0,
    EthBlockNumber = 1, // eth_blockNumber
    EthGetBlockByNumber = 2, // eth_getBlockByNumber
}

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
            _=> panic!("Api Call Does not exist")
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            ApiCall::EthBlockNumber => "EthBlockNumber".to_owned(),
            ApiCall::EthGetBlockByNumber => "EthGetBlockByNumber".to_owned(),
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
