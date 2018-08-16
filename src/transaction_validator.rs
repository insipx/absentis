//! Warning! Uses Etherscan
use log::*;
use evmap::ShallowCopy; // unsafe!
use futures::stream::Stream;
use futures::future::Future;
use futures::{Poll};
use futures::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded};
use rayon::prelude::*;
use web3::{Web3, Transport, BatchTransport};
use web3::types::{Trace, TraceFilterBuilder, Block, Transaction, TransactionId, BlockId, BlockNumber, TransactionReceipt, Log, H256, H160, FilterBuilder};
use web3::transports::batch::{BatchFuture, Batch};
use web3::api::Namespace;
use serde_derive::{Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use failure::Error;
use evmap::{ReadHandle, WriteHandle};
use super::utils;
use super::client::{self, Client};
use super::config_file::ConfigFile;
use super::err::TransactionValidatorError;
use super::filter::{self, TraceFilterCall};

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

// A transaction and all associated information
#[derive(Debug, Clone, PartialEq)]
struct Tx {
    transaction: Option<Transaction>,
    receipt: Option<TransactionReceipt>,
    traces: Option<Vec<Trace>>,
    logs: Option<Log>
}

//TODO: skip DOS transactions (blocks 2283440 -- 2718436 with > 250 traces) #p1
//https://medium.com/@tjayrush/defeating-the-ethereum-ddos-attacks-d3d773a9a063
//
impl TransactionValidator  {
    //creates a new validator from genesis to specified block
    pub fn new<T>(client: &mut Client<T>, csv_file: PathBuf, to_block: Option<BlockNumber>, to_address: H160) -> Result<Self, TransactionValidatorError>
    where
        T: BatchTransport + Send + Sync + 'static
    {
        let mut rdr = csv::Reader::from_path(csv_file.as_path())?;
        let (mut read, mut write) = evmap::new();
        for result in rdr.deserialize() {
            let res: TxEntry = result?;
            write.insert(res.block_num, MapEntry {transaction_index: res.transaction_index, location: res.location});
        }
        write.refresh();
        // discard write handle; we should never modify the original CSV
        let to_block = to_block.unwrap_or(BlockNumber::Latest);
        Ok(TransactionValidator {
            read_handle: read,
            transactions: Self::build_local_cache(client, to_block.clone(), to_address)?,
            to_block,
            to_addr: to_address,
        })
    }

    fn build_local_cache<T>(client: &mut Client<T>, to_block: BlockNumber, to_addr: H160) -> Result<HashMap<H256, Tx>, TransactionValidatorError>
        where
            T: BatchTransport + Send + Sync + 'static,
    {
        let mut cache = HashMap::new();
        let log_filter = FilterBuilder::default().address(vec![to_addr.clone()]).to_block(to_block.clone()).from_block(BlockNumber::Earliest).build();
        info!("Gathering logs");
        let fut = client.web3.eth().logs(log_filter);
        let log_vec = try_web3!(client.run(fut));
        let mut logs: HashMap<H256, Log> = HashMap::new();
        logs.extend(log_vec.into_iter().map(|l| (l.transaction_hash.unwrap(), l)));
        info!("Gathering Traces");

        let mut fut = filter::trace_filter(client, Some(to_block), to_addr);
        let mut transactions = client.run(fut).expect("Error is empty; qed");
        transactions.dedup_by_key(|x| x.transaction_hash);

        for (idx, t) in transactions.iter_mut().enumerate() {
            let tx_hash = t.transaction_hash.ok_or_else(||TransactionValidatorError::CouldNotFind(verb_msg!("Tx Hash")))?;

            if t.subtraces >= 1 && t.subtraces <= 250 {
                info!("Transaction: {:?}", t);
                client.web3_batch.eth().transaction(TransactionId::Hash(tx_hash));
                client.web3_batch.eth().transaction_receipt(tx_hash);
                client.web3_batch.trace().transaction(tx_hash);
                let mut log = None;
                if logs.contains_key(&tx_hash) {
                    log = Some(logs.get(&tx_hash).expect("Scope is conditional; qed").clone());
                }
                cache.insert(tx_hash, Tx {logs: log, traces: None, receipt: None, transaction: None });
            } else { // subtraces == 0
                info!("Transaction: {:?}", t);
                client.web3_batch.eth().transaction(TransactionId::Hash(tx_hash));
                client.web3_batch.eth().transaction_receipt(tx_hash);
                let traces = t.clone();
                let mut log = None;
                if logs.contains_key(&tx_hash) {
                    log = Some(logs.get(&tx_hash).expect("Scope is conditional; qed").clone());
                }
                cache.insert(tx_hash, Tx {logs: log, traces: Some(vec![traces]), transaction: None, receipt: None });
            }
        }

        enum TxPart {
            Receipt(TransactionReceipt),
            Transaction(Transaction),
            Trace(Vec<Trace>)
        };

        let identify_type = |v: serde_json::value::Value| {
            if v.is_array() { // traces
                Ok(TxPart::Trace(serde_json::from_value::<Vec<Trace>>(v)?))
            } else if v.is_object() && v.get("input").is_some() { // Transaction
                Ok(TxPart::Transaction(serde_json::from_value::<Transaction>(v)?))
            } else if v.is_object() && v.get("status").is_some() { // transaction receipt
                Ok(TxPart::Receipt(serde_json::from_value::<TransactionReceipt>(v)?))
            } else {
                Err(TransactionValidatorError::FailedToBuildLocalCache)
            }
        };

        info!("Submitting last batch!");
        let fut = client.web3_batch.transport().submit_batch();
        let batch = try_web3!(client.run(fut));

        for tx_part in batch.into_iter().map(|x| {
            let x:serde_json::value::Value = try_web3!(x);
            identify_type(x)
        }) {
            match tx_part? {
                TxPart::Receipt(receipt) => {
                    let tx_hash = receipt.transaction_hash;
                    let entry = try!(cache
                                         .get_mut(&tx_hash)
                                         .ok_or_else(||TransactionValidatorError::CouldNotFind(verb_msg!("Tx Hash {} in cache", tx_hash))));
                    entry.receipt = Some(receipt);
                },
                TxPart::Transaction(tx) => {
                    let tx_hash = tx.hash;
                    let entry =
                        cache
                            .get_mut(&tx_hash)
                            .ok_or_else(||TransactionValidatorError::CouldNotFind(verb_msg!("Tx Hash {} in cache", tx_hash)))?;
                    entry.transaction = Some(tx);
                },
                TxPart::Trace(traces) => {
                    let tx_hash = traces
                        .get(0).ok_or_else(||TransactionValidatorError::CouldNotFind(verb_msg!("Trace 0 {:?}", traces)))?
                        .transaction_hash.ok_or_else(||TransactionValidatorError::CouldNotFind(verb_msg!("TX Hash in traces {:?}", traces)))?;

                    let entry = cache
                        .get_mut(&tx_hash)
                        .ok_or_else(||TransactionValidatorError::CouldNotFind(verb_msg!("Tx Hash {} in cache", tx_hash)))?;
                    entry.traces = Some(traces);
                }
            }
        }

        Ok(cache)
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


#[cfg(test)]
mod tests {
    use super::*;
    use web3::types::Address;
    use crate::conf::Configuration;

    fn tx_validator(client: &mut Client::<web3::transports::http::Http>) -> TransactionValidator {
        match TransactionValidator::new(client,
                                       PathBuf::from("/home/insi/Projects/absentis/txs.csv"),
                                       Some(BlockNumber::Number(2_300_000)),
                                       Address::from("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359"))
        {
            Err(e) => {
                error!("{}", e);
                panic!("failed due to ERROR");
            },
            Ok(v) => {
                info!("Len: {}", v.read_handle.len());
                info!("Success!");
                v
            }
        }
    }

    #[test]
    fn it_should_create_new_validator() {
        pretty_env_logger::try_init();
        let conf = Configuration::new().expect("Could not create configuration");
        let mut client = Client::<web3::transports::http::Http>::new_http(&conf).expect("Could not build client");
        let validator = tx_validator(&mut client);
        info!("Validator tx: {:#?}", validator.transactions);
    }
/*
    #[test]
    fn it_should_test_cache() {
        info!("IN FILTER");
        pretty_env_logger::try_init();
        let conf = Configuration::new().expect("Could not create configuration");
        let tx_validator = TransactionValidator::new(
            PathBuf::from("/home/insi/Projects/absentis/txs.csv"),
            Some(BlockNumber::Number(1000000)),
            Address::from("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359")).unwrap();

        info!("Transaction validator instantiated");
        let mut client = Client::<web3::transports::http::Http>::new_http(&conf).expect("Could not build client");
        match tx_validator.build_local_cache(&client) {
            Err(e) => {
                error!("Error: {:?}", e);
                panic!("ERROR");
            },
            Ok(v) => {
                info!("CACHE: {:?}", v);
            }
        }
        client.turn();
    }
     */
    /*
    #[test]
    fn test_filter() {
        pretty_env_logger::try_init();
        let conf = Configuration::new().expect("Could not create configuration");
        let mut client = Client::<web3::transports::http::Http>::new_http(&conf).expect("Could not build client");
        let fut = trace_filter(&client, Some(BlockNumber::Number(2_000_000)), H160::from("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359"));
        client.ev_loop().run(fut).unwrap();
    }
    */
/*
    #[test]
    fn local_cache() {
        pretty_env_logger::try_init();
        let conf = Configuration::new().expect("Could not create configuration");
        let mut client = Client::<web3::transports::http::Http>::new_http(&conf).expect("Could not build client");
        let validator = tx_validator(&client);
        info!("CACHE: {:?}", validator.transactions);

    }
  */
}
