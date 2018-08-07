use log::*;
use failure::*;
use futures::stream::Stream;
use futures::future::{self, Future};
use futures::{Poll, Async};
use ethereum_types::Address;
use web3::{transports, BatchTransport};
use web3::types::{BlockNumber, BlockId, Transaction, Block};
use web3::helpers::CallResult;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::thread;
use super::err::TransactionFinderError;
use super::client::Client;

// -- going to need `__getBlockByNumber
// -- going to need `getLogs`
// finds all transactions associated with an address
struct TransactionFinder {
    address: Address,
    to_block: BlockNumber,
    from_block: BlockNumber,
    transactions: Arc<Mutex<VecDeque<Transaction>>>, //push_back to add, pop_front to take
}


impl TransactionFinder {
    /// get all transactions for an account from a block to a block
    /// defaults:
    /// fromBlock: latest,
    /// toBlock: latest,
    pub fn new(address: Address, to_block: Option<BlockNumber>, from_block: Option<BlockNumber>) -> Self {
        
        let t_block = to_block.unwrap_or(BlockNumber::Latest);
        let f_block = from_block.unwrap_or(BlockNumber::Latest);

        TransactionFinder {
            address,
            to_block: to_block.expect("Added default value; qed"),
            from_block: from_block.expect("Added default value; qed"),
            transactions: Arc::new(Mutex::new(VecDeque::new()))
        }
    }

    pub fn crawl<T>(self, client: &Client<T>) 
        -> Result<Crawl, TransactionFinderError> where T: BatchTransport + Sync + Send
    {   
        let remote = client.remote();
        let handle = client.handle();
        let tx_queue = self.transactions.clone();
        let addr = self.address.clone();
        let latest = || { 
            let b = client.web3.eth().block_number().wait();
            let b = match b {
                Ok(v) => v,
                Err(e) => {
                    pretty_err!("{}{}", "Could not get latest block: ", e.description());
                    if let Some(bt) = e.backtrace() {
                        error!("Backtrace: {:?}", bt);
                    }
                    panic!("Shutting down...");
                }

            };
            Ok(b.as_u64())
        };

        let (to, from) = match (self.to_block, self.from_block) {
            (BlockNumber::Latest, BlockNumber::Latest) => (latest()?, latest()?),
            (BlockNumber::Latest, BlockNumber::Earliest) => (latest()?, 0 as u64),
            (BlockNumber::Earliest, BlockNumber::Earliest) => (0 as u64, 0 as u64),
            (_,_) => Err(TransactionFinderError::ImpossibleTo)?
        };

        if from > to {
            return Err(TransactionFinderError::ImpossibleTo);
        }
        let eth = client.web3.eth();
        handle.spawn_send(future::lazy(move || {
            let eth = eth.clone();
            let block_task = |b: Block<Transaction>, tx_queue: Arc<Mutex<VecDeque<Transaction>>>, address| {
                for t in b.transactions.iter() {
                    if t.to.is_some() && t.to.unwrap() == address {
                        let mut _guard = tx_queue.lock()
                            .expect("Should never fail while holding lock");
                        (*_guard).push_back(t.clone());
                    }
                }
            };


            for i in from..to {
                eth.block_with_txs(BlockId::Number(BlockNumber::Number(i))).then(|b: Result<Block<Transaction>, web3::error::Error>| {
                    let blk = match b {
                        Ok(v) => v,
                        Err(e) => {
                            pretty_err!("{}{}{}", "Could not get block: {:x} due to: {}", i.to_string(), e.description());
                            if let Some(bt) = e.backtrace() {
                                error!("Backtrace: {:?}", bt);
                            }
                            panic!("Shutting down...");
                        }
                    };
                    remote.spawn( |_| {
                        block_task(blk, tx_queue.clone(), addr);
                        Ok(())
                    });
                    Ok(())
                }).map_err(|e: web3::error::Error | error!("Something bad happened"));
            }
            futures::future::ok(())
        }));

        Ok(Crawl { inner: self.transactions.clone() })
        // spawn tokio for every block where transaction != 0 to search for tx
        // if tx.to matches self.address, add to VecDeque
    }
}


pub struct Crawl {
    inner: Arc<Mutex<VecDeque<Transaction>>>,
}

impl Stream for Crawl {
    type Item = Transaction;
    type Error = TransactionFinderError;

    fn poll(self: &mut Self) -> Poll<Option<Self::Item>, Self::Error> {
        let mut _guard = self.inner.lock().unwrap();
        Ok(Async::Ready((*_guard).pop_front()))
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_crawl() {
        let client = Client::new
    
    }

}
*/
