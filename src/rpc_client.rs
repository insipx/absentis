//! Asynchronous JSON-RPC clients for use with Infura, and Ethereum Nodes (Geth, Parity, Etc)
mod infura_client;
mod err;
mod api_call;
mod jsonrpc_object;
mod response_object;
mod tests;

use self::response_object::ResponseObject;
use failure::Error;
use futures::Future;


pub use self::infura_client::InfuraClient;

// not all methods are defined on the client
// just the ones needed for the bounty 
pub trait EthRpcClient {
    fn getBlockNumber(&self) -> Box<dyn Future<Item=ResponseObject, Error=Error> + Send>;
    fn getBlockByNumber(&self, _: u64, _: bool ) -> Box<dyn Future<Item=ResponseObject, Error=Error> + Send>;
}


