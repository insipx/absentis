use log::*;
use failure::*;
use futures::stream::Stream;
use futures::future::{self, Future};
use futures::{Poll, Async};
use futures::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded};
use tokio_core::reactor::Handle;
use ethereum_types::Address;
use web3::{transports, BatchTransport};
use web3::types::{BlockNumber, BlockId, Transaction, Block};
use web3::helpers::CallResult;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::thread;
use super::err::TransactionFinderError;
use super::client::Client;
use super::types::MAX_BATCH_SIZE;

// -- going to need `__getBlockByNumber
// -- going to need `getLogs`
// finds all transactions associated with an address
struct TransactionFinder {
    address: Address,
    to_block: BlockNumber,
    from_block: BlockNumber,
}


impl TransactionFinder {
    /// get all transactions for an account from a block to a block
    /// defaults:
    /// fromBlock: latest,
    /// toBlock: latest,
    pub fn new(address: Address, from_block: Option<BlockNumber>, to_block: Option<BlockNumber>) -> Self {
        
        let t_block = to_block.unwrap_or(BlockNumber::Latest);
        let f_block = from_block.unwrap_or(BlockNumber::Latest);

        TransactionFinder {
            address: address,
            to_block: t_block,
            from_block: f_block,
        }
    }

    pub fn crawl<T>(self, client: &Client<T>) 
        -> Result<Crawl, TransactionFinderError>
        where 
            T: BatchTransport + Send + Sync,
            <T as web3::BatchTransport>::Batch: Send
            // <T as web3::BatchTransport>: Send + Sync
    {   
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
            b.as_u64()
        };

        let (to, from) = match (self.to_block, self.from_block) {
            (BlockNumber::Latest, BlockNumber::Latest) => (latest(), latest()),
            (BlockNumber::Latest, BlockNumber::Earliest) => (latest(), 0 as u64),
            (BlockNumber::Earliest, BlockNumber::Earliest) => (0 as u64, 0 as u64),
            (BlockNumber::Number(t), BlockNumber::Number(f)) => (t, f),
            (BlockNumber::Latest, BlockNumber::Number(f)) => (latest(), f),
            (BlockNumber::Number(t), BlockNumber::Earliest) => (t, 0 as u64),
            (_,_) => Err(TransactionFinderError::ImpossibleTo)?
        };
        pretty_info!("{}{}{}{}", "Crawling Transactions from Block: ", format_num!(from).underline(), " To Block: ", format_num!(to).underline());
        if from > to {
            return Err(TransactionFinderError::ImpossibleTo);
        }
        
        let addr = self.address.clone();
        let (tx, rx): (UnboundedSender<Transaction>, UnboundedReceiver<Transaction>) = unbounded();

        for i in from..=to {
            client.web3_batch.eth().block_with_txs(BlockId::Number(BlockNumber::Number(i)));

            if i % MAX_BATCH_SIZE == 0 || i == to { 
                let txx = tx.clone();
                let batch = client.web3_batch.transport().submit_batch().then(move |blks| {
                    pretty_info!("{}", "Batch submitted");
                    
                    let blks: Vec<Block<Transaction>> = match blks {
                        Err(e) => panic!("Error querying block: {}", e),
                        Ok(v) => {
                            v.into_iter().map(|b| match serde_json::from_value(b.unwrap()) {
                                Err(e) => {
                                    error!("{}", e);
                                    panic!("Panic due to error; could not deserialize serde_json::value::Value");
                                },
                                Ok(vv) => vv
                            }).collect()
                        },
                    };
                    blks.into_iter().for_each(|blk| {
                        blk.transactions.iter()
                            .filter(|t| t.to.is_some() && t.to.unwrap() == addr)
                            .for_each(|t| txx.unbounded_send(t.clone()).unwrap());
                    });
                    Ok(())
                });
                client.handle().spawn_send(batch);
            }
        }
        Ok(Crawl { inner: rx })
    }
}


pub struct Crawl {
    inner: UnboundedReceiver<Transaction>
}

impl Stream for Crawl {
    type Item = Transaction;
    type Error = ();

    fn poll(self: &mut Self) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner.poll()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::conf::Configuration;
    use web3::transports::http::Http;
    #[test]
    fn test_crawl() {
        let conf = Configuration::from_default().expect("Should be ok if test passes");
        let addr = Address::from("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359");
        let mut client = Client::<web3::transports::http::Http>::new_http(&conf).expect("Could not build client");
        let txs = TransactionFinder::new(addr, Some(BlockNumber::Number(500000)), Some(BlockNumber::Number(1000000)))
            .crawl(&client).expect("Could not construct stream")
            .for_each(|tx| {
                info!("TX: {:?}", tx);
                Ok(())
            });
        client.run(txs);
    }
}
