use log::*;
use evmap::ShallowCopy; // unsafe!
use futures::stream::Stream;
use futures::future::Future;
use futures::{Poll};
use futures::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded};
use rayon::prelude::*;
use web3::{Web3, Transport, BatchTransport};
use web3::types::{Trace, TraceFilterBuilder, Block, Transaction, BlockId, BlockNumber, TransactionReceipt, Log, H256, H160};
use web3::transports::batch::{BatchFuture, Batch};
use web3::api::Namespace;
use serde_derive::{Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use failure::Error;
use evmap::{ReadHandle, WriteHandle};
use super::utils::latest_block;
use super::client::{self, Client};
use super::config_file::ConfigFile;
use super::err::TransactionValidatorError;

#[derive(Deserialize, Debug, Clone)]
pub struct TxEntry  {
    #[serde(rename = "blockNum")]
    block_num: u64,
    #[serde(rename = "transactionIndex")]
    transaction_index: u32,
    location: String,
}
#[derive(Debug, Clone)]
struct MapEntry  {
    transaction_index: u32,
    location: String,
}

impl PartialEq for MapEntry {
    fn eq(&self, other: &MapEntry) -> bool {
        self.transaction_index == other.transaction_index &&
            self.location == other.location
    }
}

impl Eq for MapEntry {}

impl evmap::ShallowCopy for MapEntry {
    unsafe fn shallow_copy(&mut self) -> Self {
        MapEntry {
            transaction_index: self.transaction_index.shallow_copy(),
            location: self.location.shallow_copy()
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidEntry {
    Missing {block_num: u32, transaction_index: u32, location: String },
    Incorrect {block_num: u32, transaction_index: u32, location: String },
}

// [ miner | from | to | input | creation | self-destruct | log_N_generator | log_N_topic_M |
// log_N_data | trace_N_from | trace_N_to | trace_N_refundAddr | trace_N_creation |
// trace_N_self-destruct | trace_N_input ]
pub struct TransactionValidator {
    read_handle: ReadHandle<u64, MapEntry>,
    to_block: BlockNumber,
    to_addr: H160,
    transactions: HashMap<H256, Tx>,
}

// A transaction and it's receipt
struct Tx {
    transaction: Transaction,
    Receipt: TransactionReceipt,
    trace: Vec<Trace>,
    logs: Vec<Log>
}
//TODO: skip DOS transactions (blocks 2283440 -- 2718436 with > 250 traces) #p1
//https://medium.com/@tjayrush/defeating-the-ethereum-ddos-attacks-d3d773a9a063
//
impl TransactionValidator  {
    //creates a new validator from genesis to specified block
    pub fn new(csv_file: PathBuf, to_block: Option<BlockNumber>, to_address: H160) -> Result<Self, TransactionValidatorError> {
        let mut rdr = csv::Reader::from_path(csv_file.as_path())?;
        let (mut read, mut write) = evmap::new();
        for result in rdr.deserialize() {
            let res: TxEntry = result?;
            write.insert(res.block_num, MapEntry {transaction_index: res.transaction_index, location: res.location});
        }
        write.refresh();
        // discard write handle; we should never modify the original CSV
        Ok(TransactionValidator {
            read_handle: read,
            to_block: to_block.unwrap_or(BlockNumber::Latest),
            to_addr: to_address,
            transactions: HashMap::new(),
        })
    }

    pub fn build_local_cache<T>(&self, client: &Client<T>) -> Result<HashMap<H256, Tx>, Error> 
        where
            T: BatchTransport + Send + Sync + 'static,
            <T as web3::BatchTransport>::Batch: Send,
    {   
        let t_filter = trace_filter(client, Some(self.to_block), self.to_addr).then(|x| {
            info!("X: {:?}", x);
            Ok(())
        });

        client.handle().spawn(t_filter);
        // client.run(t_filter).unwrap();
        Ok(HashMap::new())
    }
    
    // attempts to validate csv list of transactions, returning any incorrectly included
    // transactions 
    // return a stream of missing, or incorrectly included transactions
    pub fn scan<T>(&self, client: &Client<T>) -> Result<Scan, Error>
        where
            T: BatchTransport + Send + Sync,
            <T as web3::BatchTransport>::Batch: Send
    {   
        let (tx, rx): (UnboundedSender<InvalidEntry>, UnboundedReceiver<InvalidEntry>) = unbounded();
        /*
        self.read_handle.for_each(|block| {
            block.par_iter().for_each(|tx| { // parallel iteration
                
            
            });
        }); */

        unimplemented!();
        Ok(Scan { inner: rx })
    }
    
    /// find transactions that were incorrectly included in the CSV
    fn find_misplaced<T>(&self, client: &Client<T>, tx: UnboundedSender<InvalidEntry>) -> Result<(), Error> 
        where
            T: BatchTransport + Send + Sync,
            <T as web3::BatchTransport>::Batch: Send
    {   
        // 
        // try to find each transaction
        Ok(())
    }

    /// creates a new filter from genesis to specified block.
    fn log_validator<T>(&self, client: &Client<T>, tx: UnboundedSender<InvalidEntry>) -> Result<(), Error>
        where
            T: BatchTransport + Send + Sync,
            <T as web3::BatchTransport>::Batch: Send
    {   
        unimplemented!();
        Ok(())
    }
    // log_N_generator
    fn receipt_validator<T>(&self, client: &Client<T>, tx: UnboundedSender<InvalidEntry>) -> Result<(), Error>
        where
            T: BatchTransport + Send + Sync,
            <T as web3::BatchTransport>::Batch: Send

    {   
        unimplemented!();
        Ok(())
    }

    fn trace_validator<T>(&self, client: &Client<T>, tx: UnboundedSender<InvalidEntry>) -> Result<(), Error>
        where
            T: BatchTransport + Send + Sync,
            <T as web3::BatchTransport>::Batch: Send

    {
        unimplemented!();
        Ok(())
    }
}

pub struct Scan {
    inner: UnboundedReceiver<InvalidEntry>
}

impl Stream for Scan {
    type Item = InvalidEntry;
    type Error = ();

    fn poll(self: &mut Self) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner.poll()
    }
}

// Request transactions asynchronously
/// Request transactions asynchronously. If a pending blocknumber is specified, defaults to `latest`
fn trace_filter<T>(client: &Client<T>, to_block: Option<BlockNumber>, to_address: H160) 
    -> impl Future<Item = Vec<Trace>, Error = ()>
    where
        T: BatchTransport,
{
    let to_block = to_block.unwrap_or(BlockNumber::Latest);
    let to_block: u64 = match to_block {
        BlockNumber::Earliest => 0,
        BlockNumber::Latest => latest_block(client),
        BlockNumber::Number(num) => num,
        BlockNumber::Pending => latest_block(client),
    };
    
    let filter = |from, to, addr| {

        client.web3_batch.trace()
            .filter(TraceFilterBuilder::default()
                        .from_block(BlockNumber::Number(from))
                        .to_block(BlockNumber::Number(to))
                        .to_address(vec![addr])
                        .build());
    };

    let mut multiplier = 2;
    let mut to_chunk = to_block / 4;
    let mut last_block = 0;
    while last_block <= to_block {
        filter(last_block,to_chunk, to_address.clone());
        last_block = to_chunk;
        to_chunk = last_block * multiplier;
        multiplier += 1;
    }
    
    client.web3_batch.transport().submit_batch().and_then(|x| {
        let new_item = x.into_iter().map(|j_val| {
            let j_val = match j_val {
                Err(e) => {
                    pretty_err!("{}, {}", "Web3 Error submitting batch!", format!("{}", e));
                    panic!("Shutting down due to web3 error in: {}, {}, {}", line!(), column!(), file!());
                },
                Ok(v) => v,
            };
            let j_val: Trace = serde_json::from_value(j_val).unwrap();
            j_val
        }).collect();
        futures::future::ok(new_item)
    }).map_err(|e|{
        pretty_err!("{}, {}", "Web3 Error submitting batch!", format!("{}", e));
        panic!("Shutting down due to web3 error in: {}, {}, {}", line!(), column!(), file!());
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use web3::types::Address;
    use crate::conf::Configuration;

    #[test]
    fn it_should_create_new_validator() {
        pretty_env_logger::try_init();
        // TODO: use current_dir #p1
        match TransactionValidator::new(PathBuf::from("/home/insi/Projects/absentis/txs.csv"), Some(BlockNumber::Number(1000000)), Address::from("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359")) {
            Err(e) => {
                error!("{}", e);
                panic!("failed due to ERROR");
            },
            Ok(v) => {
                info!("Len: {}", v.read_handle.len());
                assert!(v.read_handle.len() > 0);
                v.read_handle.for_each(|k, v| {
                    info!("K: {:?}, V: {:?}", k, v);
                }); 
                info!("Success!");
            }
        }
    }

    #[test]
    fn it_should_validate() {
        info!("IN FILTER");
        pretty_env_logger::try_init();
        let conf = Configuration::new().expect("Could not create configuration");
        let tx_validator = TransactionValidator::new(
            PathBuf::from("/home/insi/Projects/absentis/txs.csv"), 
            Some(BlockNumber::Number(1000000)), 
            Address::from("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359 ")).unwrap();

        info!("Transaction validator instantiated");
        let mut client = Client::<web3::transports::http::Http>::new_http(&conf).expect("Could not build client");
        tx_validator.build_local_cache(&client).unwrap();
        client.turn();
        ()
    }
}
