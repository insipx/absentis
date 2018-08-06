use failure::Fail;

#[derive(Fail, Debug)]
pub enum ConfigurationError {
    #[fail(display = "Could not find home directory. Try setting the $PATH variable")]
    CouldNotFindHomeDir,
    #[fail(display = "Invalid Toml")]
    InvalidToml(#[fail(cause)] toml::de::Error),
    #[fail(display = "Input/Output Error")]
    IOError(#[fail(cause)] std::io::Error),
    #[fail(display = "{} not found!", _0)]
    NotFound(String),
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

#[derive(Fail, Debug)]
pub enum ClientError {
    #[fail(display = "Must Specify a {}", _0)]
    MustSpecify(String),
}

