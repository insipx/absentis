//! Warning! Uses Etherscan
mod cache;
mod simpledb;
pub mod err;

use log::*;
use serde_derive::Deserialize;
use failure::Error;
use rayon::prelude::*;
use futures::{
    future::Future,
    stream::Stream,
    Poll,
    sync::mpsc::{self, UnboundedSender, UnboundedReceiver},
};
use web3::{
    BatchTransport,
    types::{Trace, Action, Res, Transaction, TransactionId, BlockNumber, TransactionReceipt, H160, BlockId, Block as Web3Block, H256, Index},
};
use std::path::PathBuf;
use super::{
    utils,
    etherscan::{EtherScan, SortType},
    client::{Client},
    err::TransactionValidatorError,
};

use self::cache::{TxType, Tx, Block, TransactionCache as Cache, CacheAction};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct TxEntry  {
    #[serde(rename = "blockNum")]
    block_num: u64,
    #[serde(rename = "transactionIndex")]
    transaction_index: usize,
    location: String,
}

impl std::fmt::Display for TxEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "block: {}, index: {}, location: {}", self.block_num, self.transaction_index, self.location)
    }
}

#[derive(Debug, Clone)]
pub enum InvalidEntry {
    Missing(H256),
    Incorrect(TxEntry, Option<H256>) // hash is None if transction does not exist on ethereum mainnet
}

impl std::fmt::Display for InvalidEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InvalidEntry::Incorrect(entry, hash) => {
                if hash.is_some() {
                    write!(f, "Incorrect Entry at {} with hash: {:x}", entry, hash.expect("scope is conditional; qed"))
                } else {
                    write!(f, "Incorrect Entry at {}, Transaction does not exist", entry)
                }
            }
            InvalidEntry::Missing(hash) => write!(f, "Missing Transaction {:x}", hash),
        }
    }
}

pub struct TransactionValidator {
    csv: Vec<TxEntry>,
    cache: Cache,
    to_block: BlockNumber,
    addr: H160,
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
        let mut csv_vec = Vec::new();
        let mut rdr = csv::Reader::from_path(csv_file.as_path())?;
        for result in rdr.deserialize() {
            let res: TxEntry = result?;
            csv_vec.push(res);
        }

        // discard write handle; we should never modify the original CSV
        let to_block = to_block.unwrap_or(BlockNumber::Latest);
        Ok(TransactionValidator {
            csv: csv_vec,
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
    pub fn scan<T>(&self, client: &Client<T>) -> Result<Scan, Error>
    where
        T: BatchTransport + Send + Sync + 'static,
        <T as web3::Transport>::Out: Send
    {
        let (tx, rx): (UnboundedSender<InvalidEntry>, UnboundedReceiver<InvalidEntry>) = mpsc::unbounded();
        self.find_misplaced(client, tx.clone())?;
        Ok(Scan { inner: rx })
    }

    /// find transactions that were incorrectly included in the CSV
    fn find_misplaced<T>(&self, client: &Client<T>, sender: UnboundedSender<InvalidEntry>) -> Result<(), Error>
    where
        T: BatchTransport + Send + Sync + 'static,
        <T as web3::Transport>::Out: Send,
    {
        let remote = client.remote();
        for entry in self.csv.par_iter() {
            let entry = entry.clone();
            if let None = self.cache.tx_by_blocknum_index(entry.block_num, entry.transaction_index) {
                let sender_async = sender.clone();
                let fut = client.web3
                    .eth()
                    .transaction(TransactionId::Block(BlockId::Number(BlockNumber::Number(entry.block_num as u64)), Index::from(entry.transaction_index)))
                    .then(move |t| {
                        let t = try_web3!(t);
                        let mut hash = None;
                        if let Some(tx) = t {
                            hash = Some(tx.hash)
                        }
                        sender_async.unbounded_send(InvalidEntry::Incorrect(entry, hash));
                        drop(sender_async);
                        Ok(())
                    });
                remote.spawn(|_| {fut});
            }
        }
        Ok(())
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
    //     - creation
    //     - data
    // in trace
    //     - self-destruct
    //     - creation
    //     - refundAddr
    //     - to
    //     - from
    //     - input
    // in transaction
    //     - input
    // in block
    //     - miner
    // [ miner | from | to | input | creation | self-destruct | log_N_generator | log_N_topic_M |
    // log_N_data | trace_N_from | trace_N_to | trace_N_refundAddr | trace_N_creation |
    // trace_N_self-destruct | trace_N_input ]
    /// does parsing to put the information we found into format of csv for easy scanning
/*
    fn mirror(&self, tx: &Tx) -> Vec<TxEntry> {
        let mut tx_vec = Vec::new();
        if tx.block.as_ref().unwrap().block.author == self.addr {
            add!("miner", tx_vec, tx)
        }
        if self.scan_bytes(&tx.transaction.as_ref().unwrap().input.0) {
            add!("input", tx_vec, tx);
        }
        if tx.transaction.as_ref().unwrap().to.is_some() && tx.transaction.as_ref().unwrap().to.unwrap() == self.addr {
            add!("to", tx_vec, tx);
        }
        if tx.transaction.as_ref().unwrap().from == self.addr {
            add!("from", tx_vec, tx);
        }
        if let Some(c_addr) = tx.receipt.as_ref().unwrap().contract_address {
            if c_addr == self.addr {
                add!("creation", tx_vec, tx)
            }
        }
        for (idx, log) in tx.receipt.as_ref().unwrap().logs.iter().enumerate() {
            if log.address == self.addr {
                let log_str = format!("{}_{}_{}", "log", idx, "generator");
                add!(log_str, tx_vec, tx);
            }
            if self.scan_bytes(&log.data.0) {
                let log_str = format!("{}_{}_{}", "log", idx, "data");
                add!(log_str, tx_vec, tx);
            }
            for (top_idx, topic) in log.topics.iter().enumerate() {
                if self.scan_bytes(&(*topic).to_vec()) {
                    let log_str = format!("{}_{}_{}_{}", "log", idx, "topic", top_idx);
                    add!(log_str, tx_vec, tx);
                }
            }
        }
        for (idx, trace) in tx.traces.as_ref().unwrap().iter().enumerate() {
            if let Some(st) = self.scan_trace_result(&trace.result) {
                if trace.trace_address.len() > 0 {
                    let trace_str = format!("{}_{}_{}_{}", "trace", idx, self.trace_addr(&trace.trace_address), st);
                    add!(trace_str, tx_vec, tx);
                } else {
                    let trace_str = format!("{}_{}_{}", "trace", idx, st);
                    add!(trace_str, tx_vec, tx)
                }
            }
            if let Some(act_vec) = self.scan_trace_action(&trace.action) {
                tx_vec.extend(act_vec.iter().map(|l| {
                    if trace.trace_address.len() > 0 {
                        TxEntry {
                            location: format!("{}_{}_{}", "trace", idx, l),
                            block_num: tx.block.as_ref().unwrap().block.number.unwrap().as_u64(),
                            transaction_index: tx.transaction.as_ref().unwrap().transaction_index.unwrap().as_u32(),
                        }
                    } else {
                        TxEntry {
                            location: format!("{}_{}_{}_{}", "trace", idx, self.trace_addr(&trace.trace_address), l),
                            block_num: trace.block_number,
                            transaction_index: tx.transaction.as_ref().unwrap().transaction_index.unwrap().as_u32(),
                        }
                    }
                }))
            }
        }

        tx_vec
    }
    */
    fn trace_addr(&self, addr: &Vec<usize>) -> String {
        let mut string = String::from("[");
        for (idx, x) in addr.iter().enumerate() {
            if idx == 0 {
                string = format!("{}{}", string, x);
            } else {
                string = format!("{}_{}", string ,x)
            }
        }
        string.push(']');
        string
    }

    fn scan_trace_result(&self, res: &Res) -> Option<String> {
        match res {
            Res::Call(_) => None,
            Res::Create(ref create_res) if create_res.address == self.addr => Some("self-destruct".into()), // might be self-destruct
            Res::FailedCallOrCreate(_) => None,
            Res::None => None,
            _ => None
        }
    }

    fn scan_trace_action(&self, action: &Action) -> Option<Vec<String>> {
        let mut act_vec: Vec<String> = Vec::new();
        match action {
            Action::Call(ref call) => {
                if call.from == self.addr {act_vec.push("from".into())}
                if call.to == self.addr { act_vec.push("to".into())}
                if self.scan_bytes(&call.input.0) { act_vec.push("input".into())}
            },
            Action::Create(create) => {
                if create.from == self.addr {act_vec.push("from".into())}
            },
            Action::Suicide(s) => {
                if s.address == self.addr {act_vec.push("creation".into())} // might be creation
                if s.refund_address == self.addr {act_vec.push("refundAddr".into())}
            },
            Action::Reward(_) => {
                // don't do rewards

            },
        }
        if act_vec.len() == 0 {
            None
        } else {Some(act_vec)}
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
                                       Some(BlockNumber::Number(1_000_000)),
                                       Address::from("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359"))
        {
            Err(e) => {
                let error = failure::Error::from(e);
                error!("{}", error.as_fail());
                error!("{:?}", error);
                panic!("failed due to ERROR");
            },
            Ok(v) => {
                info!("Len: {}", v.csv.len());
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
        let _validator = tx_validator(&mut client);
    }

    #[test]
    fn it_should_scan() {
        pretty_env_logger::try_init();
        let conf = Configuration::new().expect("Could not create configuration");
        let mut client = Client::<web3::transports::http::Http>::new_http(&conf).expect("Could not build client");
        let validator = tx_validator(&mut client);
        let fut = validator.scan().unwrap().for_each(|inv| {
            info!("Invalid Transaction: {}", inv);
            Ok(())
        });
        client.run(fut);
    }

    #[test]
    fn it_should_mirror() {
        pretty_env_logger::try_init();
        let conf = Configuration::new().expect("Could not create configuration");
        let mut client = Client::<web3::transports::http::Http>::new_http(&conf).expect("Could not build client");
        let validator = tx_validator(&mut client);
        let tx = validator.cache.tx_by_blocknum(988725).unwrap();
        let mir = validator.mirror(&tx);
        info!("Mirrored: {:?}", mir);
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
