//! Warning! Uses Etherscan
mod cache;
mod simpledb;
pub mod err;

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
    types::{Trace, Action, Res, Transaction, TransactionId, BlockNumber, TransactionReceipt, H160, BlockId, Block as Web3Block, H256},
};
use std::path::PathBuf;
use super::{
    utils,
    etherscan::{EtherScan, SortType},
    client::{Client},
    err::TransactionValidatorError,
};

use self::cache::{TxType, Tx, Block, TransactionCache as Cache, CacheAction};

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

pub struct TransactionValidator {
    read_handle: ReadHandle<u64, MapEntry>,
    cache: Cache,
    to_block: BlockNumber,
    addr: H160,
}

macro_rules! add {
    ($location: expr, $vec: ident, $tx: ident) => ({
        $vec.push(TxEntry {
            block_num: $tx.block.as_ref().unwrap().block.number.unwrap().as_u64(),
            transaction_index: $tx.transaction.as_ref().unwrap().transaction_index.unwrap().as_u32(),
            location: $location.into()
        })
    });
}

//TODO: skip DOS transactions (blocks 2283440 -- 2718436 with > 250 traces) #p1
//https://medium.com/@tjayrush/defeating-the-ethereum-ddos-attacks-d3d773a9a063
//
impl TransactionValidator  {
    //creates a new validator from genesis to specified block
    pub fn new<T>(client: &mut Client<T>, csv_file: PathBuf, to_block: Option<BlockNumber>, address: H160)
                  -> Result<Self, TransactionValidatorError>
    where
        T: BatchTransport + Send + Sync + 'static
    {
        let mut rdr = csv::Reader::from_path(csv_file.as_path())?;
        let (read, mut write) = evmap::new();
        for result in rdr.deserialize() {
            let res: TxEntry = result?;
            write.insert(res.block_num, MapEntry {transaction_index: res.transaction_index, location: res.location});
        }
        write.refresh();
        // discard write handle; we should never modify the original CSV
        let to_block = to_block.unwrap_or(BlockNumber::Latest);
        Ok(TransactionValidator {
            read_handle: read,
            cache: Self::build_local_cache(client, to_block.clone(), address)?,
            to_block,
            addr: address,
        })
    }

    fn build_local_cache<T>(client: &mut Client<T>, to_block: BlockNumber, addr: H160)
                            -> Result<Cache, TransactionValidatorError>
    where
        T: BatchTransport + Send + Sync + 'static,
    {
        let mut cache = Cache::new(addr, BlockNumber::Earliest, to_block)?;
        if cache.is_populated() {
            return Ok(cache);
        }

        info!("gathering transactions from EtherScan");
        let eth_scan = EtherScan::new();
        let to_block = utils::as_u64(client, to_block);
        let hashes = eth_scan.get_tx_by_account(client.ev_loop(), addr, 0, to_block, SortType::Ascending)?;

        // gather these in three asynchronous calls. This works best if the node being used
        // allows for 3+ threads for the RPC
        let (txs, receipts, traces, blocks) = (client.batch(), client.batch(), client.batch(), client.batch());

        let mut block_numbers: Vec<(H256, u64)> = Vec::new();
        for (hash, block_num) in hashes.iter() {
            txs.eth().transaction(TransactionId::Hash(*hash));
            receipts.eth().transaction_receipt(*hash);
            traces.trace().transaction(*hash);
            // only get block number if we haven't gotten it yet
            // associates a TXHash with a blocknumber. important later when we create Block{} struct
            // make seperate storage for blocks
            // deserialize into Bincode/Message pack and compress
            match block_numbers.binary_search_by_key(block_num, |&(_, blk_num)| blk_num) {
                Ok(_) => {}, //ele already exists
                Err(pos) => {
                    block_numbers.insert(pos, (*hash, *block_num));
                    blocks.eth().block(BlockId::Number(BlockNumber::Number(*block_num)));
                }
            }
        }

        let (sender, receiver) = mpsc::unbounded();
        let txs = cache_get_task::<Transaction, _, _>(txs.transport(), sender.clone(), |val| {TxType::from(val)}); // .then here
        let receipts = cache_get_task::<TransactionReceipt, _, _>(receipts.transport(), sender.clone(), |val|{TxType::from(val)});
        let traces = cache_get_task::<Vec<Trace>, _, _>(traces.transport(), sender.clone(), |val| {TxType::from(val)});

        let blocks = cache_get_task::<Web3Block<H256>, _, _>(blocks.transport(), sender, move |val| {
            let pos = &block_numbers
                .binary_search_by_key(&val.number.expect(&verb_msg!("block num should not be pending")).as_u64(), |(_, blk_num)| *blk_num).unwrap();
            let (tx, _) = block_numbers.get(*pos).unwrap().clone();
            TxType::from(Block { tx_hash: tx, block: val })
        });

        client.handle().spawn(txs);
        client.handle().spawn(receipts);
        client.handle().spawn(traces);
        client.handle().spawn(blocks);

        let fut = receiver.for_each(|tx_type| {
            cache.insert(tx_type);
            Ok(())
        });

        info!("Submitting batch requests of Transactions, Receipts, and Traces");
        client.run(fut).unwrap();
        info!("Finished building local cache. Saving...");
        cache.save()?;
        Ok(cache)
    }

    // attempts to validate csv list of transactions, returning any incorrectly included
    // transactions
    // return a stream of missing, or incorrectly included transactions
    pub fn scan(&self) -> Result<Scan, Error> {
        let (tx, rx): (UnboundedSender<InvalidEntry>, UnboundedReceiver<InvalidEntry>) = mpsc::unbounded();
        self.find_misplaced(tx.clone());
        Ok(Scan { inner: rx })
    }

    /// find transactions that were incorrectly included in the CSV
    fn find_misplaced(&self, sender: UnboundedSender<InvalidEntry>) -> Result<(), Error>
    {

        /*
        self.read_handle.for_each(|(k, entry)| {

        });
        Ok(())
         */
        unimplemented!();
    }
/*
    fn transaction_validator(&self, transaction: Transaction, tx: &UnboundedSender<InvalidEntry>) -> Result<(), Error>
    {
        unimplemented!();
    }
*/

    // log_N_generator
    /// finds addresses in transaction receipt
    fn receipt_validator(&self, receipt: TransactionReceipt, tx: &UnboundedSender<InvalidEntry>) -> Result<(), Error> {
        unimplemented!();
    }

    /// returns true if an addr is contained within a vector of traces
    fn trace_validator(&self, traces: Vec<Trace>, tx: &UnboundedSender<InvalidEntry>) -> Result<(), Error> {
        unimplemented!();
    }

    /// returns true if the addr is contained within a series of bytes
    fn scan_bytes(&self, bytes: &Vec<u8>) -> bool {
        bytes.windows(self.addr.len()).position(|window| &(*self.addr) == window).is_some()
    }

    // in Logs/Receipt
    //     - log_N_topic_M
    //     - log_N_generator
    // in trace
    //     - self-destruct
    //     - creation
    //     - refundAddr
    //     - to
    //     - from
    // in transaction
    //     - input
    // in block
    //     - miner
    // [ miner | from | to | input | creation | self-destruct | log_N_generator | log_N_topic_M |
    // log_N_data | trace_N_from | trace_N_to | trace_N_refundAddr | trace_N_creation |
    // trace_N_self-destruct | trace_N_input ]
    /// does parsing to put the information we found into format of csv for easy scanning

    fn mirror(&self, tx: Tx) -> Vec<TxEntry> {
        let mut tx_vec = Vec::new();
        if tx.block.as_ref().unwrap().block.author == self.addr {
            add!("miner", tx_vec, tx)
        }
        if self.scan_bytes(&tx.transaction.as_ref().unwrap().input.0) {
            add!("input", tx_vec, tx);
        }
        if tx.transaction.as_ref().unwrap().to.unwrap() == self.addr {
            add!("to", tx_vec, tx);
        }
        if tx.transaction.as_ref().unwrap().from == self.addr {
            add!("from", tx_vec, tx);
        }
        for (idx, trace) in tx.traces.unwrap().iter().enumerate().skip(1) {

        }

        tx_vec
    }

    fn scan_trace_result(&self, res: Res) -> Option<String> {
        match res {
            Res::Call(ref call_res) if self.scan_bytes(&call_res.output.0) => Some("data".into()),
            Res::Create(ref create_res) if self.scan_bytes(&create_res.code.0) => Some("data".into()),
            Res::Create(ref create_res) if create_res.address == self.addr => Some("creation".into()),
            Res::FailedCallOrCreate(msg) => None,
            Res::None => None,
            _ => None
        }
    }

    fn scan_trace_action(&self, action: Action) -> Option<String> {
        None
    }
}

/// asynchronously send a batch request
// decides on conversion through the predicate F
fn cache_get_task<A, T, F>(batch: &web3::transports::Batch<T>, sender: UnboundedSender<TxType>, fun: F) -> impl Future<Item=(), Error=()>
where
    A: serde::de::DeserializeOwned,
    T: BatchTransport,
    F: Fn(A) -> TxType,
{
    batch.submit_batch()
        .from_err::<TransactionValidatorError>()
        .and_then(|vals| {
            let res = vals.into_iter()
                .map(|val| {
                    serde_json::from_value(try_web3!(val)).map_err(|e| e.into())
                })
                .collect::<Result<Vec<A>, TransactionValidatorError>>();
            futures::future::result(res)
        }).and_then(move |vals| {
            match vals
                .into_iter()
                .try_for_each(|val| sender.unbounded_send(fun(val))) {
                    Err(e) => error!("{}", verb_msg!("{}", e)),
                    Ok(_) => (),
                };
            drop(sender);
            Ok(())
        }).map_err(|e| {
            error!("{}", verb_msg!("{}", e));
            error!("{:?}", e);
            panic!("Shutting down");
        })
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
                                       PathBuf::from("/home/insi/Projects/absentis/tx_list.csv"),
                                       Some(BlockNumber::Number(6_000_000)),
                                       Address::from("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359"))
        {
            Err(e) => {
                let error = failure::Error::from(e);
                error!("{}", error.as_fail());
                error!("{:?}", error);
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
    }
/*
    #[test]
    fn it_should_scan() {
        pretty_env_logger::try_init();
        let conf = Configuration::new().expect("Configuration creation failed");
        let mut client = Client::<web3::transports::http::Http>::new_http(&conf).expect("Could not build client");
        let validator = tx_validator(&mut client);
        validator.scan();

    }
    */
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
