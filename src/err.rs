use failure::Fail;

#[derive(Fail, Debug)]
pub enum ClientError {
    #[fail(display = "Must Specify a {}", _0)]
    MustSpecify(String),
}

#[derive(Fail, Debug)]
pub enum TransactionFinderError {
    #[fail(display = "Impossible to find transactions from a later block to an earlier block!")]
    ImpossibleTo,
    #[fail(display = "A Web3 Error Occured: {}, Backtrace: {}", _0, _1)]
    Web3(String, String),
}

impl From<web3::error::Error> for TransactionFinderError {
    fn from(err: web3::error::Error) -> TransactionFinderError {
        TransactionFinderError::Web3(err.description().to_string(), format!("{:#?}", err.backtrace()))
    }
}


#[derive(Fail, Debug)]
pub enum TransactionValidatorError {
    #[fail(display = "CSV parsing failed: {}", _0)]
    CSV(csv::Error),
    #[fail(display = "Could not ascertain type of Transaction Part from Batch Request in order to build local cache")]
    FailedToBuildLocalCache,
    #[fail(display = "Error deserializing JSON: {}", _0)]
    FailedToDecode(serde_json::Error),
    #[fail(display = "Could not find: {}", _0)]
    CouldNotFind(String),
    #[fail(display = "{}", _0)]
    Etherscan(super::etherscan::EtherScanError),
    #[fail(display = "Web3 Error Occured {}", _0)]
    Web3(String),
    #[fail(display = "Cache Error {}", _0)]
    Cache(#[cause] crate::transaction_validator::err::CacheError),
}

impl From<crate::transaction_validator::err::CacheError> for TransactionValidatorError {
    fn from(err: crate::transaction_validator::err::CacheError) -> TransactionValidatorError {
        TransactionValidatorError::Cache(err)
    }
}

impl From<csv::Error> for TransactionValidatorError {
    fn from(err: csv::Error) -> TransactionValidatorError {
        TransactionValidatorError::CSV(err)
    }
}

impl From<serde_json::Error> for TransactionValidatorError {
    fn from(err: serde_json::Error) -> TransactionValidatorError {
        TransactionValidatorError::FailedToDecode(err)
    }
}

impl From<super::etherscan::EtherScanError> for TransactionValidatorError {
    fn from(err: super::etherscan::EtherScanError) -> TransactionValidatorError {
        TransactionValidatorError::Etherscan(err)
    }
}

impl From<web3::error::Error> for TransactionValidatorError {
    fn from(err: web3::error::Error) -> TransactionValidatorError {
        TransactionValidatorError::Web3(format!("{}", err))
    }
}
