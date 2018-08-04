//! Asynchronous JSON-RPC clients for use with Infura, and Ethereum Nodes (Geth, Parity, Etc)
mod infura_client;
mod err;
mod api_call;
mod jsonrpc_object;
mod response_object;
mod tests;

use ethereum_types::{Address};

use self::response_object::ResponseObject;
use failure::Error;
use futures::Future;
use crate::ethereum_objects::BlockString;


pub use self::infura_client::InfuraClient;

// not all methods are defined on the client
// just the ones needed for the bounty 
pub trait EthRpcClient { //eth_ namespace
    fn block_number(&self) -> Box<dyn Future<Item=ResponseObject, Error=Error> + Send>;
    fn get_block_by_number(&self, _: u64, _: bool ) -> Box<dyn Future<Item=ResponseObject, Error=Error> + Send>;
    fn gas_price(&self) -> Box<dyn Future<Item=ResponseObject, Error=Error> + Send>;
    fn get_balance(&self, _: Address, _: Option<usize>, _: Option<BlockString>) -> Box<dyn Future<Item=ResponseObject, Error=Error> + Send>;
}


