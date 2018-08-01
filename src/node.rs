use log::*;
use failure::*;
use ethereum_types::Address;
use std::io::Write;
use hyper::{Client, Uri as HyperUri, Method, Request};
use hyper::rt::{self, Future, Stream};
use hyper_tls::HttpsConnector;
use hyper::header::HeaderValue;
use crate::conf::Configuration;
use crate::err::NodeError;
use crate::types::*;
use crate::json_builder::JsonBuilder;

// -- going to need `__getBlockByNumber
// -- going to need `getLogs`
// finds all transactions associated with an address
struct TransactionFinder {
    address: Address,
    toBlock: u64,
    fromBlock: u64,
}


impl TransactionFinder {
    /// get all transactions for an account from a block to a block
    /// defaults:
    /// fromBlock: latest,
    /// toBlock: latest,
    pub fn new(address: Address, to_block: Option<u64>, from_block: Option<u64>) -> Self {
        
        // let t_block = to_block.unwrap_or(get_latest_block());
        // let f_block = from_block.unwrap_or(get_latest_block());

        TransactionFinder {
            address,
            toBlock: to_block.unwrap(),
            fromBlock: from_block.unwrap(),
        }
    }
}

pub fn get_latest_block(conf: &Configuration) -> Result<impl Future<Item=(), Error=()>, Error>  {
    let https = HttpsConnector::new(4)?;
    let client = Client::builder().build::<_, hyper::Body>(https);
    let api_key = conf.api_key();
    let uri = build_request_uri(api_key)?;
    let data = JsonBuilder::default().method("eth_blockNumber").build()?;
    let mut req = Request::new(hyper::Body::from(data));
    *req.method_mut() = Method::POST;
    *req.uri_mut() = uri.clone().into();
    // Error for `HeaderValue` was private, could not make own error type. So this line can panic
    req.headers_mut().insert("Content-Type", HeaderValue::from_str("application/json")?);


    Ok(client.request(req).and_then(|res| {
        println!("POST: {}", res.status());
        res.into_body().for_each(|chunk| {
            std::io::stdout().write_all(&chunk)
                .map_err(|e| panic!("Example expects stdout"))
        })
    }).map(|res| {
        println!("Something");
    }).map_err(|err| {
        panic!("TODO");
    }))
}

fn build_request_uri(api_key: String) -> Result<Uri, NodeError> {
    let full_str = format!("{}{}", MAINNET_AUTHORITY, api_key);
    let uri: hyper::Uri = full_str.parse()?;
    //uri.map_err(|e| NodeError::UrlParseError(e))
    Ok(uri.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::sync::{Once, ONCE_INIT};
    use env_logger;

    static INIT: Once = ONCE_INIT;
    fn setup() {
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    #[test]
    fn it_should_build_uri() {
        let conf = Configuration::from_default().expect("Configuration error");
        let re = Regex::new(r"https://mainnet.infura.io/[a-zA-Z0-9]{32}").expect("Regex creation failed");
        let uri: Uri = match build_request_uri(conf.api_key()){
            Ok(u) => u.into(),
            Err(e) => {
                error!("Error: {}", e);
                panic!("Failed due to error");
            }
        };

        println!("URI: {:#?}", String::from(uri.clone()));
        assert!(re.is_match(&String::from(uri.clone())));
    }

    #[test]
    fn it_should_get_the_latest_block() {
        //pub fn get_latest_block(conf: Configuration) -> Result<(), Error>  {
        let conf = Configuration::from_default().expect("Configuration error");
        debug!("API_KEY: {}", conf.api_key());
        let res = rt::run(get_latest_block(&conf).expect("TODO"));
/*
        match res {
            Ok(v) => {
                debug!("RES: {:#?}", v);
            },
            Err(e) => {
                error!("Error: {}", e);
                error!("API_KEY: {}", conf.api_key());
                error!("Cause: {:#?}", e.cause());
                panic!("Test failed due to error");
            }
        }
        */
    }
}
