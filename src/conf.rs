// macros
use log::*;
use failure::Fail;
use serde_derive::*;
// structs
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use failure::Error;

#[derive(Fail, Debug)]
pub enum ConfigurationError {
    #[fail(display = "Could not find home directory. Try setting the $PATH variable")]
    CouldNotFindHomeDir,
    #[fail(display = "Invalid Toml")]
    InvalidToml(#[fail(cause)] toml::de::Error),
    #[fail(display = "Input/Output Error")]
    IOError(#[fail(cause)] std::io::Error),
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    infura_api_key: String,
}

pub trait Parse {
    fn parse(&self) -> Result<Configuration, toml::de::Error>;
}

impl Parse for String {
    fn parse(&self) -> Result<Configuration, toml::de::Error> {
        toml::from_str(self)
    }
}


impl Configuration {
    pub fn empty() -> Configuration {
        Configuration {
            infura_api_key: "".to_string(),
        }
    }
    
    /// create a new default configuration at ~/.config/absentis.toml
    pub fn new_default() -> Result<Configuration, Error> {
        let empty_config = Self::empty();
        let config_path = Self::default_path()?;
        let mut file = fs::File::create(config_path.as_path())?;
        let toml = toml::to_string_pretty(&empty_config)?;
        file.write_all(toml.as_bytes())?;
        Ok(empty_config)
    }

    pub fn from_default() -> Result<Configuration, Error> {
        let path = Self::default_path()?;
        fs::read_to_string(path.as_path())?.parse().map_err(|e| ConfigurationError::InvalidToml(e).into())
    }

    fn default_path() -> Result<PathBuf, ConfigurationError> {
        dirs::config_dir().and_then(|mut conf| {
            conf.push("absentis.toml");
            Some(conf)
        }).ok_or(ConfigurationError::CouldNotFindHomeDir)
    }
}


impl Configuration {
    pub fn api_key(&self) -> String {
        self.infura_api_key.clone()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;
   /* this test tends to screw things up
    #[test]
    fn it_should_create_new_default_config() {
        setup();
        let conf = Configuration::new_default().expect("Could not create new default configuration");
        debug!("Empty Config: {:?}", conf);
    }
*/
    #[test]
    fn it_should_return_default_path() {
        env_logger::try_init();
        let path = Configuration::default_path();
        let path = match path {
            Ok(p) => p,
            Err(e) => {
                error!("Error in test: {}", e);
                panic!("Failed due to error");
            }
        };
        // TODO: change to make general test #p2 
        assert_eq!(path.to_str().unwrap(), "/home/insi/.config/absentis.toml");
    }

    #[test]
    fn it_should_return_config_from_default_path() {
        env_logger::try_init();
        let conf = Configuration::from_default();
        match conf {
            Ok(c) =>  {
                info!("Config: {:?}", c);
            },
            Err(e) => {
                error!("Error in test: {}", e);
                error!("Cause: {:#?}", e.cause());
                error!("Trace: {:#?}", e.backtrace());
                panic!("Failed due to error");
            }
        }
    }
}
