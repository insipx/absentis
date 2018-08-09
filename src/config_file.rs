// macros
use serde_derive::*;
use log::{log, info, error, debug};
// structs
use std::fs;
use std::io::Write;
use std::env;
use std::path::PathBuf;
use std::collections::HashMap;
use failure::Error;
use config::{self, File, Config};
use reduce::Reduce;
use super::err::ConfigurationError;
use super::types::INFURA_URL;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
    nodes: Option<Vec<EthNode>>,
    infura: Option<Infura>,
    default_transport: Option<Transport>
}

#[derive(Serialize, Deserialize, Debug)]
enum Transport {
    Http,
    Ipc
}

#[derive(Serialize, Deserialize, Debug)]
struct EthNode {
    #[serde(rename = "identifier")]
    ident: String,
    http: Option<Http>,
    ipc: Option<Ipc>
}

impl std::fmt::Display for EthNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.ident)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Http {
    url: String,
    port: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Ipc {
    path: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Infura {
    api_key: String,
}


impl Default for ConfigFile {

    fn default() -> Self {

        let mut nodes: Vec<EthNode> = Vec::new();
        nodes.push(EthNode {
            ident: "Parity".to_string(),
            http: Some(Http {
                url: "http://localhost".to_string(),
                port: 8545 as usize
            }),
            ipc: None,
        });

        let infura = Some(Infura {
            api_key: "".to_string(),
        });

        ConfigFile {
            nodes: Some(nodes),
            infura,
            default_transport: Some(Transport::Http)
        }
    }
}
// file creation
impl ConfigFile {
    /// Default configuration path is ~/.config/absentis.toml (On UNIX)
    /// this can be modified by passing -c (--config) to absentis
    pub fn new(mut config_path: Option<PathBuf>) -> Result<Self, Error> {
        let mut tmp = env::temp_dir();
        tmp.push("absentis_default.toml");
        info!("Temp Config Path: {:?}", &tmp);
        let mut default_file = fs::File::create(tmp.clone())?;
        let default_config = Self::default();
        let toml = toml::to_string_pretty(&default_config)?;
        default_file.write_all(toml.as_bytes())?;
        info!("Default ConfigFile: {:?}", default_config);

        if config_path.is_none() { // if a custom configuration path has not been set, use default
            config_path = Some(Self::default_path().and_then(|p| { 
                if !p.as_path().exists() { // check to make sure the user config exists, 
                    let mut new_f = fs::File::create(p.as_path())?; // if not create an empty file so we can fill it with defaults
                    new_f.write_all(toml.as_bytes())?;
                }
                Ok(p)
            })?);
        }
        let mut conf = Config::new();
        conf.merge(File::with_name(tmp.to_str().expect("Temp file should always be valid UTF-8")))?;
        conf.merge(
                File::with_name(config_path.expect("Scope is conditional; qed")
                                .to_str()
                                .ok_or_else(|| ConfigurationError::InvalidConfigPath)?
                )
            )?;

        // info!("Configuration: {:?}", conf.try_into::<HashMap<String, String>>()?);
        conf.try_into().map_err(|e| e.into())
    }
    
    pub fn from_default() -> Result<ConfigFile, Error> {
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

// getters
impl ConfigFile {

    fn infura_url(&self) -> Result<String, ConfigurationError> {
        Ok(format!("{}{}", INFURA_URL, self.infura_key()?))
    }
    pub fn infura_key(&self) -> Result<String, ConfigurationError>  {
        let inf = self.infura.as_ref()
            .ok_or_else(||ConfigurationError::NotFound("Infura Api Key".to_string()))?;
        Ok(inf.api_key.clone())
    }
    
    // returns the url from the first Eth node that matches the predicate function
    pub fn url<F>(&self, fun: F) -> Result<String, ConfigurationError> 
        where 
            F: Fn(&EthNode) -> bool
    {
        let nodes = self.nodes
            .as_ref()
            .ok_or_else(|| ConfigurationError::OptionNotSet("Eth Nodes".to_string()))?; 
        let node: Option<EthNode> = nodes.iter().filter(|x| fun(x)).take(1).reduce(|e| e);
        

        // panic because predicate entered should never be incorrect
        // that would be an internal bug
        let node = node.expect("Predicate entered should never be incorrect; qed");
        
        let http: &Http = node.http.as_ref()
            .ok_or_else(|| ConfigurationError::OptionNotSet(format!("HTTP info for node {}", node)))?;
        Ok(format!("{}:{}", http.url, http.port))
    }
    
    // returns the ipc path from the first EthNode that matches the predicate function
    pub fn ipc_path<F>(&self, fun: F) -> Result<PathBuf, ConfigurationError> 
        where 
            F: Fn(&EthNode) -> bool
    {
        let nodes = self.nodes
            .as_ref()
            .ok_or_else(|| ConfigurationError::OptionNotSet("Eth Nodes".to_string()))?;
        
        let path: Option<PathBuf> = None;

        let node: Option<EthNode> = nodes.iter().filter(|x| fun(x)).take(1).reduce(|e| e);
            
        // panic because predicate entered should never be incorrect
        // that would be an internal bug
        let node = node.expect("Predicate entered should never be incorrect; qed");
        
        let ipc: &Ipc = node.ipc.as_ref()
            .ok_or_else(|| ConfigurationError::OptionNotSet(format!("IPC info for node {}", node)))?;

        Ok(PathBuf::new(ipc.path));
    }
}

pub trait Parse {
    fn parse(&self) -> Result<ConfigFile, toml::de::Error>;
}

impl Parse for String {
    fn parse(&self) -> Result<ConfigFile, toml::de::Error> {
        toml::from_str(self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, error, info, log};
    use env_logger;
    // this test tends to screw things up
/*
    #[test]
    fn it_should_create_new_default_config() {
        env_logger::try_init();
        let conf = Configuration::new(None); 

        match conf {
            Ok(v) => {
                info!("Default Config: {:?}", v);
            }, 
            Err(e) => {
                error!("Error: {}", e);
                panic!("Failed due to error");
            }
        }
    }
*/
    #[test]
    fn it_should_return_default_path() {
        env_logger::try_init();
        let path = ConfigFile::default_path();
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
        let conf = ConfigFile::from_default();
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
