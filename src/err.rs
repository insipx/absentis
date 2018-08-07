use failure::Fail;

#[derive(Fail, Debug)]
pub enum ConfigurationError {
    #[fail(display = "Could not find home directory. Try setting the $PATH variable")]
    CouldNotFindHomeDir,
    #[fail(display = "Invalid Toml")]
    InvalidToml(#[fail(cause)] toml::de::Error),
    #[fail(display = "Error decoding configuration {}", _0)]
    DecodeError(#[fail(cause)] toml::ser::Error),
    #[fail(display = "Input/Output Error")]
    IOError(#[fail(cause)] std::io::Error),
    #[fail(display = "{} not found!", _0)]
    NotFound(String),
    #[fail(display = "Invalid path for config file; not a valid UTF-8 String!")]
    InvalidConfigPath,
    #[fail(display = "Option Not Set: {}", _0)]
    OptionNotSet(String),

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
}
