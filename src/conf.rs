//! parses configuration file and CLI options to return configured values
use super::config_file::{ConfigFile, EthNode};

enum Node {
    Infura,
    EthNode
}

pub struct Configuration {
    file: PathBuf,
    node: Node,
}

impl Configuration {
    
    // get a configured client
    get_client() -> Client {
        unimplemented!();    
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

}


impl From<ConfigFile> for Configuration {
    fn from(conf: ConfigFile) -> Configuration {
        
    }
}
