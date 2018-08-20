//! Warning! Uses Etherscan
mod cache;
use log::*;
use evmap::{/*unsafe!*/ ShallowCopy /*unsafe!*/, ReadHandle};
use serde_derive::Deserialize;
use failure::Error;
use futures::{
    future::Future,
    stream::Stream,
    Poll,
    sync::mpsc::{self, UnboundedSender, UnboundedReceiver},
};
use web3::{
    BatchTransport,
    types::{Trace, Transaction, TransactionId, BlockNumber, TransactionReceipt, H160, FilterBuilder},
};
use std::path::PathBuf;
use super::{
    utils,
    etherscan::{EtherScan, SortType},
    client::{Client},
    err::TransactionValidatorError,
};

use self::cache::{TxType, TransactionCache as Cache};

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
    cache: Cache,
    to_block: BlockNumber,
    to_addr: H160,
}

//TODO: skip DOS transactions (blocks 2283440 -- 2718436 with > 250 traces) #p1
//https://medium.com/@tjayrush/defeating-the-ethereum-ddos-attacks-d3d773a9a063
//
impl TransactionValidator  {
    //creates a new validator from genesis to specified block
    pub fn new<T>(client: &mut Client<T>, csv_file: PathBuf, to_block: Option<BlockNumber>, to_address: H160)
                  -> Result<Self, TransactionValidatorError>
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
            cache: Self::build_local_cache(client, to_block.clone(), to_address)?,
            to_block,
            to_addr: to_address,
        })
    }

    fn build_local_cache<T>(client: &mut Client<T>, to_block: BlockNumber, to_addr: H160)
                            -> Result<Cache, TransactionValidatorError>
    where
        T: BatchTransport + Send + Sync + 'static,
    {
        let mut cache = Cache::new();

        info!("gathering logs");
        let log_filter = FilterBuilder::default() // top-level logs not needed. Included in TransactionReceipt
            .address(vec![to_addr.clone()])
            .to_block(to_block.clone())
            .from_block(BlockNumber::Earliest)
            .build();
        let fut = client.web3.eth().logs(log_filter);
        let logs = try_web3!(client.run(fut));
        cache.extend(logs);

        info!("gathering transactions from EtherScan");
        let eth_scan = EtherScan::new();
        let to_block = utils::as_u64(client, to_block);
        let mut transactions = eth_scan.get_tx_by_account(client.ev_loop(), to_addr, 0, to_block, SortType::Ascending)?;

        let (txs, receipts, traces) = (client.batch(), client.batch(), client.batch());

        for (idx, t) in transactions.iter_mut().enumerate() {
            let tx_hash = t.hash;
            info!("transaction: {:?}", t);
            txs.eth().transaction(TransactionId::Hash(tx_hash));
            receipts.eth().transaction_receipt(tx_hash);
            traces.trace().transaction(tx_hash);
        }

        let (sender, receiver) = mpsc::unbounded();
        let txs = txs.transport().submit_batch()
            .from_err::<TransactionValidatorError>()
            .and_then(|txs| {
                let res = txs.into_iter()
                    .map(|tx| serde_json::from_value::<Transaction>(try_web3!(tx)).map_err(|e| e.into()))
                    .collect::<Result<Vec<Transaction>, TransactionValidatorError>>();
                futures::future::result(res)
            }).and_then(|txs| {
                let sender = sender.clone();
                txs.into_iter()
                    .try_for_each(|tx| sender.unbounded_send(TxType::Transaction(tx)));
                Ok(())
            }).map_err(|e| {
                error!("{}", verb_msg!("{}", e));
            });
        let receipts = receipts.transport().submit_batch()
            .from_err::<TransactionValidatorError>()
            .and_then(|receipts| {
                let res = receipts.into_iter()
                    .map(|rec| serde_json::from_value::<TransactionReceipt>(try_web3!(rec)).map_err(|e| e.into()))
                    .collect::<Result<Vec<TransactionReceipt>, TransactionValidatorError>>();
                futures::future::result(res)
            }).and_then(|recs| {
                let sender = sender.clone();
                recs.into_iter()
                    .try_for_each(|rec| sender.unbounded_send(TxType::Receipt(rec)));
                Ok(())
            }).map_err(|e| {
                error!("{}", verb_msg!("{}", e));
            });

        let traces = traces.transport().submit_batch()
            .from_err::<TransactionValidatorError>()
            .and_then(|traces| {
                let res = traces.into_iter()
                    .map(|trace| serde_json::from_value::<Vec<Trace>>(try_web3!(trace)).map_err(|e| e.into()))
                    .collect::<Result<Vec<Vec<Trace>>, TransactionValidatorError>>();
                futures::future::result(res)
            }).and_then(|traces| {
                let sender = sender.clone();
                traces
                    .into_iter()
                    .try_for_each(|trace| sender.unbounded_send(TxType::Traces(trace)));
                Ok(())
            }).map_err(|e| { // handle error
                error!("{}", verb_msg!("{}", e));
            });

        client.handle().spawn(txs);
        client.handle().spawn(receipts);
        client.handle().spawn(traces);

        let fut = receiver.for_each(|tx_type| {
            cache.insert(tx_type);
            Ok(())
        });

        info!("Submitting last batch requests of Transactions, Receipts, and Traces!");
        client.run(fut);
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
        let (tx, rx): (UnboundedSender<InvalidEntry>, UnboundedReceiver<InvalidEntry>) = mpsc::unbounded();
        /*
        self.read_handle.for_each(|block| {
            block.par_iter().for_each(|tx| { // parallel iteration


            });
        }); */

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

    fn tx_validator(client: &mut Client<web3::transports::http::Http>) -> TransactionValidator {
        match TransactionValidator::new(client,
                                       PathBuf::from("/home/insi/Projects/absentis/txs.csv"),
                                       Some(BlockNumber::Number(2_300_000)),
                                       Address::from("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359"))
        {
            Err(e) => {
                error!("{}", e);
                error!("{:?}", failure::Error::from(e).cause());
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
        info!("Validator tx: {:#?}", validator.cache);
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
