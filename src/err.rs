use failure::Fail;

#[derive(Fail, Debug)]
pub enum ConfigurationError {
    #[fail(display = "Could not find home directory. Try setting the $PATH variable")]
    CouldNotFindHomeDir,
    #[fail(display = "Invalid Toml: {}", _0)]
    InvalidToml(#[fail(cause)] toml::de::Error),
    #[fail(display = "Error serializing configuration: {}", _0)]
    DecodeError(#[fail(cause)] toml::ser::Error),
    #[fail(display = "Input/Output Error")]
    IOError(#[fail(cause)] std::io::Error),
    #[fail(display = "{} not found!", _0)]
    NotFound(String),
    #[fail(display = "Invalid path for config file; not a valid UTF-8 String!")]
    InvalidConfigPath,
    #[fail(display = "Option Not Set: {}", _0)]
    OptionNotSet(String),
    #[fail(display = "Generation failed; Default configuration file already exists!")]
    ConfigExists,
    #[fail(display = "Config Error occurred")]
    Config(#[cause] config::ConfigError),
}

impl From<config::ConfigError> for ConfigurationError {
    fn from(err: config::ConfigError) -> ConfigurationError {
        ConfigurationError::Config(err)
    }
}

impl From<std::io::Error> for ConfigurationError {
    fn from(err: std::io::Error) -> ConfigurationError {
        ConfigurationError::IOError(err)
    }
}

impl From<toml::de::Error> for ConfigurationError {
    fn from(err: toml::de::Error) -> ConfigurationError {
        ConfigurationError::InvalidToml(err)
    }
}

impl From<toml::ser::Error> for ConfigurationError {
    fn from(err: toml::ser::Error) -> ConfigurationError {
        ConfigurationError::DecodeError(err)
    }
}

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
