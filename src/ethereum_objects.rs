mod hex;
mod block;
mod transaction;
mod block_string;
pub use self::block::Block;
pub use self::transaction::Transaction;
pub use self::hex::Hex;
pub use self::block_string::BlockString;


pub enum EthObjType {
    Hex(Hex),
    Block(Block),
    Transaction(Transaction),
}

impl From<EthObjType> for String {
    fn from(val: EthObjType) -> String {
        match val {
            EthObjType::Hex(_) => "Hex".to_string(),
            EthObjType::Block(_) => "Block".to_string(),
            EthObjType::Transaction(_) => "Transaction".to_string(),
        }
    }
}
