//! Asynchronous JSON-RPC clients for use with Infura, and Ethereum Nodes (Geth, Parity, Etc)
#[macro_use] mod utils;
mod infura_client;
mod api_call;
mod request_object;
mod response_object;
mod tests;
pub mod err;

use ethereum_types::{Address, H256};
use self::response_object::ResponseObject;
use failure::Error;
use futures::Future;
use crate::ethereum_objects::{BlockString, Hex, Block, Transaction, EthObjType};


pub use self::infura_client::InfuraClient;

// not all methods are defined on the client
// just the ones needed for the bounty 
pub trait EthRpcClient { //eth_ namespace
    fn block_number(&self) -> Box<dyn Future<Item=Hex, Error=Error> + Send>;
    fn gas_price(&self) -> Box<dyn Future<Item=Hex, Error=Error> + Send>;
    fn get_balance(&self, _: Address, _: Option<usize>, _: Option<BlockString>) -> Box<dyn Future<Item=Hex, Error=Error> + Send>;
    fn get_block_by_hash(&self, _: H256) 
        -> Box<Future<Item=Block, Error=Error> + Send>;
    fn get_block_by_number(&self, _: u64) -> Box<dyn Future<Item=Block, Error=Error> + Send>;

}


