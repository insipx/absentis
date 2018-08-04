//! Asynchronous JSON-RPC clients for use with Infura, and Ethereum Nodes (Geth, Parity, Etc)
use log::*;
use failure::*;
use colored::Colorize;
use futures::{Future};
use hyper::{Client, Uri as HyperUri, Method, Request};
use hyper::rt::{Stream};
use hyper_tls::HttpsConnector;
use hyper::client::{HttpConnector, ResponseFuture};
use hyper::header::HeaderValue;
use crate::types::*;
use crate::conf::Configuration;
use crate::ethereum_objects::{ResponseObject};
use crate::json_builder::JsonBuilder;
use crate::err::RpcError;
use crate::utils::IntoHexStr;

// not all methods are defined on the client
// just the ones needed for the bounty 
pub trait EthRpcClient {
    fn getBlockNumber(&self) -> Box<dyn Future<Item=ResponseObject, Error=Error> + Send>;
    fn getBlockByNumber(&self, _: u64, _: bool ) -> Box<dyn Future<Item=ResponseObject, Error=Error> + Send>;
}

pub struct InfuraClient {
    conf: Configuration,
    client: hyper::client::Client<HttpsConnector<HttpConnector>, hyper::Body>,
    uri: Uri
}

impl InfuraClient  {
    pub fn new() -> Result<Self, Error> {
        let conf = Configuration::from_default()?;
        let https = HttpsConnector::new(4)?;
        let client = Client::builder().build::<_, hyper::Body>(https);
        let api_key = conf.api_key();
        let uri = Self::build_request_uri(api_key)?; 
        Ok(
          InfuraClient {
            conf, client, uri
          }
        )
    }

    fn post_request(&self, json: String) -> ResponseFuture {
        let mut req = Request::new(hyper::Body::from(json));
        *req.method_mut() = Method::POST;
        *req.uri_mut() = self.uri();
        
        req.headers_mut().insert("Content-Type", HeaderValue::from_static(JSON_APP_HEADER));
        self.client.request(req)
    }

    fn do_post(&self, json: String) -> impl Future<Item = ResponseObject, Error = Error> {
        self.post_request(json)
            .and_then(|res| {
                assert_eq!(res.status(), hyper::StatusCode::OK);
                res.into_body().concat2()
            }).map_err(|e| e.into())
            .and_then(|json| {
                futures::future::result(ResponseObject::from_bytes(json.into_bytes()).map_err(|e| e.into()))
            })
    }

    fn do_get(&self, json: String) -> ResponseFuture {
        self.client.get(self.uri())
    }

    fn uri(&self) -> HyperUri {
        self.uri.clone().into()
    }

    fn build_request_uri(api_key: String) -> Result<Uri, RpcError> {
        let full_str = format!("{}{}", MAINNET_AUTHORITY, api_key);
        let uri: hyper::Uri = full_str.parse()?;
        //uri.map_err(|e| NodeError::UrlParseError(e))
        Ok(uri.into())
    }
}

// TODO build a better macro that is clearer #p3
macro_rules! rpc_call {
    ($call:ident, $sel: ident, $params: expr) => ({
        match JsonBuilder::default()
            .method(ApiCall::$call)
            .params($params.to_vec())
            .build()
            .map_err(|e| futures::future::err(e.into())) 
        {
            Ok(j) => Box::new($sel.do_post(j)),
            Err(e) => Box::new(e)
        }
    })
}
// write a serializer
impl EthRpcClient for InfuraClient {
    fn getBlockNumber(&self) -> Box<dyn Future<Item=ResponseObject, Error = Error> + Send> {
        return rpc_call!(EthBlockNumber, self, []);
    }

    fn getBlockByNumber(&self, block_num: u64, show_tx_details: bool
        ) -> Box<Future<Item=ResponseObject, Error = Error> + Send> 
    {
        return rpc_call!(EthGetBlockByNumber, self, [serde_json::Value::String(block_num.into_hex_str()), serde_json::Value::Bool(show_tx_details)]);
    }
}

// TODO
// for use directly with an ethereum node (e.g. Parity)
/*
pub struct NodeClient {


}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use log::*;
    use regex::Regex;
    use std::sync::{Once, ONCE_INIT};
    use env_logger;
    
    #[test]
    fn it_should_get_the_latest_block() {
        env_logger::try_init();
        //pub fn get_latest_block(conf: Configuration) -> Result<(), Error>  {
        let client = InfuraClient::new().expect("Error building client!");
        
        let task = client.getBlockNumber().map_err(|err: failure::Error| { 
            error!("ERROR: {:?}", err);
            error!("ERROR: {:?}", err.cause());
            error!("Backtrace: {:?}", err.backtrace());
            panic!("Failed due to error");
        }).and_then(|res| {
            info!("{}: {:?}","eth_blockNumber".green().bold(), res);
            assert_eq!(res.to_str(), "EthBlockNumber");
            Ok(())
        });
        
        let mut rt = tokio::runtime::Runtime::new().expect("Could not construct tokio runtime");
        rt.block_on(task);
    }

    #[test]
    fn it_should_get_a_block_by_number() {
        env_logger::try_init();
        //pub fn get_latest_block(conf: Configuration) -> Result<(), Error>  {
        let client = InfuraClient::new().expect("Error building client!");
        
        let task = client.getBlockByNumber(300, true).map_err(|err: failure::Error| { 
            error!("ERROR: {:?}", err);
            error!("ERROR: {:?}", err.cause());
            error!("Backtrace: {:?}", err.backtrace());
            panic!("Failed due to error");
        }).and_then(|res| {
            info!("{}: {:?}","eth_getBlockByNumber".green().bold(), res);
            assert_eq!(res.to_str(), "EthGetBlockByNumber");
            assert!(res.)
            Ok(())
        });
        let mut rt = tokio::runtime::Runtime::new().expect("Could not construct tokio runtime");
        rt.block_on(task);
    }

    #[test]
    fn it_should_build_uri() {
        env_logger::try_init();
        let conf = Configuration::from_default().expect("Configuration error");
        let re = Regex::new(r"https://mainnet.infura.io/[a-zA-Z0-9]{32}").expect("Regex creation failed");
        let uri: Uri = match InfuraClient::build_request_uri(conf.api_key()){
            Ok(u) => u.into(),
            Err(e) => {
                error!("Error: {}", e);
                panic!("Failed due to error");
            }
        };

        info!("URI: {:#?}", String::from(uri.clone()));
        assert!(re.is_match(&String::from(uri.clone())));
    }
}
