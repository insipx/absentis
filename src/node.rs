use log::*;
use failure::*;
use ethereum_types::Address;

// -- going to need `__getBlockByNumber
// -- going to need `getLogs`
// finds all transactions associated with an address
struct TransactionFinder {
    address: Address,
    toBlock: u64,
    fromBlock: u64,
}


impl TransactionFinder {
    /// get all transactions for an account from a block to a block
    /// defaults:
    /// fromBlock: latest,
    /// toBlock: latest,
    pub fn new(address: Address, to_block: Option<u64>, from_block: Option<u64>) -> Self {
        
        // let t_block = to_block.unwrap_or(get_latest_block());
        // let f_block = from_block.unwrap_or(get_latest_block());

        TransactionFinder {
            address,
            toBlock: to_block.unwrap(),
            fromBlock: from_block.unwrap(),
        }
    }
}
