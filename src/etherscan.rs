#[macro_use] mod types;
pub use self::Types::{EtherScanTx};
use std::collections::HashMap;
use failure::Fail;

pub struct EtherScan;

pub enum SortType {
    Ascending,
    Descending,
    None
}

impl EtherScan {
    fn get_tx_by_account(addr: H256, from: u64, to: u64, sort: SortType) -> Result<Vec<EtherScanTx>,  > {
        let response = match sort {
            Ascending => reqwest::get(types::tx_list!(addr, from, to, "asc")),
            Descending => reqwest::get(types::tx_list!(addr, from, to, "des")),
            None => reqwest::get(types::tx_list!(addr, from, to))
        }?;
        serde_json::from_str::<Vec<EtherScanTx>>(response.text()?)?
    }
}


#[derive(Fail, Debug)]
pub enum EtherScanError {
    #[fail(display = "Could not decode Etherscan Response: {}", _0)]
    FailedToDecode(serde_json::Error),
}

impl From<serde_json::Error> for EtherScanError {
    fn from(err: serde_json::Error) -> EtherScanError {
        EtherScanError::FailedToDecode(err)
    }
}
