//! A transaction cache for transaction_validator
use std::collections::HashMap;
use web3::types::{Transaction, TransactionReceipt, Trace, Log, H256};

#[derive(Debug)]
pub struct TransactionCache(HashMap<H256, Tx>);

/// A transaction and all associated information (Transaction, Receipt, Traces, Extra Logs)
#[derive(Debug, Clone, PartialEq)]
struct Tx {
    transaction: Option<Transaction>,
    receipt: Option<TransactionReceipt>,
    traces: Option<Vec<Trace>>,
    logs: Option<Log>
}

#[derive(Debug, Clone, PartialEq)]
pub enum TxType {
    Transaction(Transaction),
    Receipt(TransactionReceipt),
    Traces(Vec<Trace>),
    Logs(Log),
}

impl Tx {
    pub fn new(transaction: Option<Transaction>, receipt: Option<TransactionReceipt>, traces: Option<Vec<Trace>>, logs: Option<Log>) -> Self {
        Tx { transaction, receipt, traces, logs }
    }
}

// TODO: Make this a generic cache for H256 hashes #p3
impl TransactionCache {
    pub fn new() -> Self {
        TransactionCache(HashMap::new())
    }

    /// Insert a TxType into Cache
    pub fn insert(&mut self, tx: impl CacheAction) {
        tx.insert(&mut self.0);  // handle errors with .exists() to make sure we're not overwriting anything
    }

    /// extend cache with a vector of CacheAction Types
    pub fn extend(&mut self, val: Vec<impl CacheAction>) {
        self.0.extend(val.into_iter().map(|x| (x.hash().clone(), x.empty())))
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
            cache.insert(self.hash().clone(), Tx { transaction: Some(self), logs: None, traces: None, receipt: None});
        }
    }

    fn exists(&self, cache: &HashMap<H256, Tx>) -> bool {
        if cache.contains_key(self.hash()) && cache.get(self.hash()).expect("scope is conditional; qed").transaction.is_some() {
            true
        } else {
            false
        }
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
        if cache.contains_key(self.hash()) && cache.get(self.hash()).expect("scope is conditional").receipt.is_some() {
            true
        } else {
            false
        }
    }

    fn empty(self) -> Tx {
        Tx {receipt: Some(self), logs: None, traces: None, transaction: None}
    }
}

impl CacheAction for Vec<Trace> {
    fn hash(&self) -> &H256 {
        &self.get(0)
            .expect(&verb_msg!("Cannot insert an empty vector!"))
            .transaction_hash
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
            if cache.contains_key(self.hash()) && cache.get(self.hash()).expect("scope is conditional; qed").traces.is_some() {
                true
            } else {
                false
            }
        }
    }

    fn empty(self) -> Tx {
        Tx {traces: Some(self), transaction: None, logs: None, receipt: None}
    }
}

impl CacheAction for Log {
    fn hash(&self) -> &H256 {
        &self.transaction_hash.expect("Transaction hash cannot be empty")
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
        if cache.contains_key(self.hash()) && cache.get(self.hash()).expect("scope is conditional; qed").logs.is_some() {
            true
        } else {
            false
        }
    }

    fn empty(self) -> Tx {
        Tx { logs: Some(self), transaction: None, traces: None, receipt: None }
    }
}
