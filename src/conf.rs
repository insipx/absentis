// macros
use log::*;
use failure::Fail;
use serde_derive::*;
// structs
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use failure::Error;
use super::err::ConfigurationError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    node: NodeType,
    transport: TransportType
}

#[derive(Serialize, Deserialize, Debug)]
enum NodeType {
    Parity{url: Option<String>, port: Option<usize>, ipc_path: Option<String> }, // url and port to Parity
    Geth{url: Option<String>, port: Option<usize>, ipc_path: Option<String> }, // url to parity node
    Infura{api_key: String} // infura API key
}
#[derive(Serialize, Deserialize, Debug)]
pub enum TransportType {
    Http,
    Ipc,
}

impl Default for Configuration {

    fn default() -> Self {
        Configuration {
            node: NodeType::Parity{url: Some("http://localhost".to_owned()), port: Some(8545) , ipc_path: None},
            transport: TransportType::Http
        }
    }
}

impl Configuration {
    
    /// create a new default configuration at ~/.config/absentis.toml
    pub fn new_default() -> Result<Configuration, Error> {
        let empty_config = Self::default();
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
    pub fn infura_key(&self) -> Result<String, ConfigurationError>  {
        match self.node {
            NodeType::Infura{api_key} => Ok(api_key),
            _ => Err(ConfigurationError::NotFound("Api Key".to_owned()))
        }
    }
    
    pub fn url(&self) -> Result<String, ConfigurationError> {
        match self.node {
            NodeType::Infura{api_key} => Ok(format!("{}{}", super::types::INFURA_URL, api_key) ),
            NodeType::Parity{url, port, ipc_path} => {
                let url = url.ok_or(ConfigurationError::NotFound("Parity Url".to_owned()));
                let port = port.ok_or(ConfigurationError::NotFound("Parity Port".to_owned()));
                Ok(format!("{}:{}", url?, port?))
            },
            NodeType::Geth{url, port, ipc_path} => {
                let url = url.ok_or(ConfigurationError::NotFound("Geth Url".to_owned()));
                let port = port.ok_or(ConfigurationError::NotFound("Geth Port".to_owned()));
                Ok(format!("{}:{}", url?, port?))
            },
        }
    }
    
    pub fn ipc_path(&self) -> Result<String, ConfigurationError> {
        match self.node {
            NodeType::Parity{url, port, ipc_path} => ipc_path.ok_or(ConfigurationError::NotFound("Parity IPC Path".into())),
            NodeType::Geth{url, port, ipc_path} => ipc_path.ok_or(ConfigurationError::NotFound("Geth IPC Path".into()))
        }
    }


    pub fn transport(&self) -> &TransportType {
        &self.transport
    }
}

pub trait Parse {
    fn parse(&self) -> Result<Configuration, toml::de::Error>;
}

impl Parse for String {
    fn parse(&self) -> Result<Configuration, toml::de::Error> {
        toml::from_str(self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;
    // this test tends to screw things up
    #[test]
    fn it_should_create_new_default_config() {
        env_logger::try_init();
        let conf = Configuration::new_default().expect("Could not create new default configuration");
        debug!("Empty Config: {:?}", conf);
    }

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
                error!("Cause: {:#?}", e.as_fail());
                error!("Trace: {:#?}", e.backtrace());
                panic!("Failed due to error");
            }
        }
    }
}
