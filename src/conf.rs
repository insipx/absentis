//! parses configuration file and CLI options to return configured values
use log::{log, error, warn};
use failure::Error;
use std::path::PathBuf;
use web3::BatchTransport;
use super::client::Client;
use super::config_file::{ConfigFile, EthNode, Transport};
use super::cli::parse;

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

        let mut file = None;
        let mut url: String; let mut log_level: LogLevel; let mut transport;
        if opts.file.is_none() && opts.url.is_some() {
            url = opts.url.expect("A single url means `node` or `infura` was used");
            file = None;
            transport = opts.transport.expect("A single url means `node` or `infura` was used");
        } else if opts.file.is_some() && opts.url.is_none() { // use defaults from file
            let file = opts.file.expect("Scope is conditional; qed");
            let default = file.default_ident();
            let url_info = file.transport(None, |node| node.matches(&default));
            if url_info.is_err() { // if we couldn't find a node to use, fallback to Infura
                error!("{}", url_info.expect_err("Scope is conditional; qed"));
                warn!("Could not find a node to use based on default identifier. \
                      attempting to fall back to Infura");
                url = file.infura_url()?;
                transport = Transport::Infura;
            } else {
                let url_info = url_info.expect("Scope is conditional; qed");
                url = url_info.0;
                transport = url_info.1;
            }
        } else {
            // everything is specified
            url = opts.url.expect("Everything specified; qed");
            file = opts.file;
            transport = opts.transport.expect("Everything specified; qed");
        }

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

// if a host is not specific, we use the first host to respond to a Http JsonRPC net_peers query
// if no hosts are found, we warn the user that no hosts have been found, and are falling back to
// infura
// if infura api key is not found, we inform the user and exit
fn get_node(file: &ConfigFile) -> EthNode {
    // first, ping the host to make sure they are even up
    // then depending on IPC or Http, create a synchronous web3 client,
    // return first host to respond to netPeers
    unimplemented!();
}
