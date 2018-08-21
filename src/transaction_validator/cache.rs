//! A transaction cache for transaction_validator
use log::*;
use serde_derive::*;
use std::{
    collections::HashMap,
    path::PathBuf,
};
use rayon::prelude::*;
use web3::types::{Transaction, TransactionReceipt, Trace, Log, H160, H256, U256, BlockNumber};
use rustbreak::{FileDatabase, deser::Bincode};
use super::err::CacheError;

/// a simple cache for storing transactions
#[derive(Debug)]
pub struct TransactionCache {
    /// the cache
    cache: HashMap<H256, Tx>,
    /// name of an object in a database in OS temporary directory.
    name: String, // -- name convention = ADDRESS_FROMBLOCK_TOBLOCK
    db: FileDatabase<HashMap<H256, Tx>, Bincode>,
}

/// A transaction and all associated information (Transaction, Receipt, Traces, Extra Logs)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Tx {
    transaction: Option<Transaction>,
    receipt: Option<TransactionReceipt>,
    traces: Option<Vec<Trace>>,
    logs: Option<Log>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum TxType {
    Transaction(Transaction),
    Receipt(TransactionReceipt),
    Traces(Vec<Trace>),
    Logs(Log),
}

impl std::fmt::Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TxType::Transaction(_) => write!(f, "Transaction"),
            TxType::Receipt(_) => write!(f, "Receipt"),
            TxType::Traces(_) => write!(f, "Traces"),
            TxType::Logs(_) => write!(f, "Logs"),
        }
    }
}

impl Tx {
    pub fn new(transaction: Option<Transaction>, receipt: Option<TransactionReceipt>, traces: Option<Vec<Trace>>, logs: Option<Log>) -> Self {
        Tx { transaction, receipt, traces, logs }
    }
}

trait Displayable {
    fn display(&self) -> String;
}
impl Displayable for BlockNumber {
    fn display(&self) -> String {
        match self {
            BlockNumber::Earliest => "earliest".to_string(),
            BlockNumber::Latest => "latest".to_string(),
            BlockNumber::Number(num) => num.to_string(),
            BlockNumber::Pending => "pending".to_string(),
        }
    }
}


// TODO: Make this a generic cache for H256 hashes #p3
impl TransactionCache {
    pub fn new(addr: H160, from_block: BlockNumber, to_block: BlockNumber) -> Result<Self, CacheError> {
        let name = format!("0x{:x}_{}_{}", addr, from_block.display(), to_block.display());
        let cache; let database;
        if let Some(db) = Self::try_local(name) {
            database = db;
            cache = db.load(true);
        } else {
            database = FileDatabase::from_file(Self::default_path()?);
            cache = HashMap::new();
        }
        TransactionCache {
            cache: HashMap::new(),
            name,
            db:
        }
    }

    /// Insert a TxType into Cache
    pub fn insert(&mut self, tx: impl CacheAction) {
        if tx.exists(&self.cache) {
            error!("Transaction already exists in cache. Aborting...");
            std::process::exit(1);
        } else {
            tx.insert(&mut self.cache);  // handle errors with .exists() to make sure we're not overwriting anything
        }
    }

    /// extend cache with a vector of CacheAction Types
    pub fn extend(&mut self, val: Vec<impl CacheAction>) {
        self.cache.extend(val.into_iter().map(|x| (x.hash().clone(), x.empty())))
    }

    pub fn tx_by_blocknum(&self, block_num: u64) -> Option<H256> {
        let block_num = U256::from(block_num);
        self.cache.par_iter()
            .find_any(|(k, v)| v.transaction.unwrap().block_number.expect("Block number will never be pending; qed") == block_num)
            .map(|(k, _)| k.clone())
            // v.transaction.block_number.expect("Block number will never be pending; qed")
    }

    /// Save all transactions to a temporary database that lives in /tmp
    pub fn save(&self) {

    }
    ///Try to find a local copy of database
    pub fn try_local(name: String) -> Option<HashMap<H256, Tx>> {
        if self.exists() {
            FileDatabase::<HashMap<H256, Tx>, Bincode>::from_file(
                Self::default_path(name).unwrap().as_path(), HashMap::new()) // TODO unwrap
        } else {
            None
        }
    }

    fn default_path(name: String) -> Result<PathBuf, CacheError> {
        dirs::cache_dir().and_then(|mut d| {
            d.push("absentis");
            d.push(name);
        }).ok_or(CacheError::NotFound("Operating System Specific Cache Directory".to_string()))?;
    }

    fn exists(name: String) -> Result<bool, CacheError> {
        let path = Self::default_path(name)?;
        path.as_path().exists()
    }
}

// TODO: make this a macro #p2
impl CacheAction for TxType {
    fn insert(self, cache: &mut HashMap<H256, Tx>) {
        match self {
            TxType::Transaction(tx) => tx.insert(cache),
            TxType::Receipt(rec) => rec.insert(cache),
            TxType::Traces(tr) => tr.insert(cache),
            TxType::Logs(logs) => logs.insert(cache),
        }
    }

    fn exists(&self, cache: &HashMap<H256, Tx>) -> bool {
        match self {
            TxType::Transaction(tx) => tx.exists(cache),
            TxType::Receipt(rec) => rec.exists(cache),
            TxType::Traces(tr) => tr.exists(cache),
            TxType::Logs(logs) => logs.exists(cache),
        }
    }

    fn hash(&self) -> &H256 {
        match self {
            TxType::Transaction(tx) => tx.hash(),
            TxType::Receipt(rec) => rec.hash(),
            TxType::Traces(tr) => tr.hash(),
            TxType::Logs(logs) => logs.hash(),
        }
    }

    fn empty(self) -> Tx {
        match self {
            TxType::Transaction(tx) => tx.empty(),
            TxType::Receipt(rec) => rec.empty(),
            TxType::Traces(tr) => tr.empty(),
            TxType::Logs(logs) => logs.empty(),
        }
    }
}

/// Common actions for web3 types in Cache
pub trait CacheAction {
    /// gets transaction hash of item in cache
    fn hash(&self) -> &H256;
    /// inserts an item into cache
    fn insert(self, cache: &mut HashMap<H256, Tx>);
    /// checks cache if this type exists within it
    fn exists(&self, cache: &HashMap<H256, Tx>) -> bool;
    /// converts CacheAction type to Tx type with all other fields of Tx as `None`
    fn empty(self) -> Tx;
}

impl CacheAction for Transaction {
    fn hash(&self) -> &H256 {
        &self.hash
    }
    fn insert(self, cache: &mut HashMap<H256, Tx>) {
        if cache.contains_key(&self.hash) {
            let entry = cache.get_mut(&self.hash()).expect("scope is conditional; qed");
            entry.transaction = Some(self);
        } else {
            cache.insert(self.hash().clone(), self.empty());
        }
    }

    fn exists(&self, cache: &HashMap<H256, Tx>) -> bool {
        cache.contains_key(self.hash()) && cache.get(self.hash()).expect("scope is conditional; qed").transaction.is_some()
    }

    fn empty(self) -> Tx {
        Tx { transaction: Some(self), logs: None, traces: None, receipt: None }
    }
}

impl CacheAction for TransactionReceipt {
    fn hash(&self) -> &H256 {
        &self.transaction_hash
    }

    fn insert(self, cache: &mut HashMap<H256, Tx>) {
        if cache.contains_key(&self.transaction_hash) {
            let entry = cache.get_mut(&self.transaction_hash).expect("scope is conditional; qed");
            entry.receipt = Some(self);
        } else {
            cache.insert(self.hash().clone(), self.empty());
        }
    }

    fn exists(&self, cache: &HashMap<H256, Tx>) -> bool {
        cache.contains_key(self.hash()) && cache.get(self.hash()).expect("scope is conditional").receipt.is_some()
    }

    fn empty(self) -> Tx {
        Tx {receipt: Some(self), logs: None, traces: None, transaction: None}
    }
}

impl CacheAction for Vec<Trace> {
    fn hash(&self) -> &H256 {
        self.get(0).as_ref()
            .expect(&verb_msg!("Cannot insert an empty vector!"))
            .transaction_hash.as_ref()
            .expect(&verb_msg!("TX hash cannot be `None`"))
    }
    fn insert(self, cache: &mut HashMap<H256, Tx>) {
        if cache.contains_key(self.hash()) {
            let entry = cache.get_mut(self.hash()).expect("scope is conditional; qed");
            entry.traces = Some(self);
        } else {
            cache.insert(self.hash().clone(), self.empty());
        }
    }

    fn exists(&self, cache: &HashMap<H256, Tx>) -> bool {
        if self.len() <= 0 {
            false
        } else {
            cache.contains_key(self.hash()) && cache.get(self.hash()).expect("scope is conditional; qed").traces.is_some()
        }
    }

    fn empty(self) -> Tx {
        Tx {traces: Some(self), transaction: None, logs: None, receipt: None}
    }
}

impl CacheAction for Log {
    fn hash(&self) -> &H256 {
        self.transaction_hash.as_ref().expect("Transaction hash cannot be empty")
    }

    fn insert(self, cache: &mut HashMap<H256, Tx>) {
        if cache.contains_key(self.hash()) {
            let entry = cache.get_mut(self.hash()).expect("scope is conditional; qed");
            entry.logs = Some(self);
        } else {
            cache.insert(self.hash().clone(), self.empty());
        }
    }

    fn exists(&self, cache: &HashMap<H256, Tx>) -> bool {
        cache.contains_key(self.hash()) && cache.get(self.hash()).expect("scope is conditional; qed").logs.is_some()
    }
    fn empty(self) -> Tx {
        Tx { logs: Some(self), transaction: None, traces: None, receipt: None }
    }
}

impl From<Transaction> for TxType {
    fn from(t: Transaction) -> TxType {
        TxType::Transaction(t)
    }
}

impl From<TransactionReceipt> for TxType {
    fn from(tr: TransactionReceipt) -> TxType {
        TxType::Receipt(tr)
    }
}

impl From<Vec<Trace>> for TxType {
    fn from(traces: Vec<Trace>) -> TxType {
        TxType::Traces(traces)
    }
}

impl From<Log> for TxType {
    fn from(log: Log) -> TxType {
        TxType::Logs(log)
    }
}
