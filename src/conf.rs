//! parses configuration file and CLI options to return configured values
use log::{log, error, warn};
use failure::Error;
use std::path::PathBuf;
use web3::BatchTransport;
use super::err::ConfigurationError;
use super::client::Client;
use super::config_file::{ConfigFile, Transport};

pub struct Configuration {
    file: Option<ConfigFile>,
    log_level: LogLevel,
    url: String,
    transport: Transport,
}
#[derive(Debug)]
pub enum LogLevel {
    None,   // Error by default
    Pleasant, // info's/warns
    Tolerable, // debug
    InsaneMode // trace/debug/info/warns
}

enum Node {
    Infura,
    EthNode
}

enum Action {
    Validate,
    // more actions
}

impl Configuration {

    pub fn new() -> Result<Self, Error> {
        let opts = super::cli::parse()?;

        let mut log_level: LogLevel;
        let (file, url, transport) = url_or_file(opts.file, opts.url, opts.transport)?;
        Ok(Configuration {
            file, url, transport,
            log_level: opts.log_level,
        })
    }

    // get a configured client
    pub fn get_client<T>(&self) -> Client<T> where T: BatchTransport {
        unimplemented!();
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn ipc_path(&self) -> PathBuf {
        PathBuf::from(&self.url)
    }
}

// TODO: #p3
// if a host is not specific, we use the first host to respond to a Http JsonRPC net_peers query
// if no hosts are found, we warn the user that no hosts have been found, and are falling back to
// infura
// if infura api key is not found, we inform the user and exit
// fn get_node(file: &ConfigFile) -> EthNode {
//     // first, ping the host to make sure they are even up
//     // then depending on IPC or Http, create a synchronous web3 client,
//     // return first host to respond to netPeers
//     unimplemented!();
// }

type UrlOrFile = (Option<ConfigFile>, String, Transport);
fn url_or_file(file: Option<ConfigFile>, url: Option<String>, transport: Option<Transport>)
               -> Result<UrlOrFile,Error> {
    match (file, url) {
        (None, Some(url)) => {
            let transport = transport.expect("A single url means `node` or `infura` was used");
            Ok((None, url, transport))
        },
        (Some(file), None) => {
            let default = file.default_ident();
            let url_info = file.transport(None, |node| node.matches(&default));
            if url_info.is_err() { // if we couldn't find a node to use, fallback to Infura
                error!("{}", url_info.expect_err("scope is conditional; qed"));
                warn!("Could not find a node to use based on default identifier. \
                       attempting to fall back to Infura");
                let url = file.infura_url()?;
                let transport = Transport::Infura;
                Ok((Some(file), url, transport))
            } else {
                let (url, transport) = url_info.expect("Scope is conditional; qed");
                Ok((Some(file), url, transport))
            }
        },
        (f @ Some(_), Some(url)) => {
            // everything is specified
            let transport = transport.expect("Everything specified; qed");
            Ok((f, url, transport))
        }
        _ => {
            error!("{}", verb_msg!("Must specify one of file or url"));
            Err(ConfigurationError::NotFound("A suitable configuration could not be found".to_string())
                .into())
        }
    }

}

