//! Asynchronous JSON-RPC clients for use with Infura, and Ethereum Nodes (Geth, Parity, Etc)
use log::*;
use failure::*;
use futures::{Poll, Async, Future};
use hyper::{Client, Uri as HyperUri, Method, Request};
use hyper::rt::{self, Stream};
use hyper_tls::HttpsConnector;
use hyper::client::{HttpConnector, ResponseFuture};
use hyper::header::HeaderValue;
use std::io::Write;
use crate::types::*;
use crate::conf::Configuration;
use crate::ethereum_objects::{ResponseObject};
use crate::json_builder::JsonBuilder;
use crate::err::RpcError;

// not all methods are defined on the client
// just the ones needed for the bounty 
pub trait EthRpcClient {
    fn getBlockNumber(&self) -> Box<dyn Future<Item=ResponseObject, Error=Error>>;
    fn getBlockByNumber(&self) -> Box<dyn Future<Item=ResponseObject, Error=Error>>;
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
                futures::future::result(ResponseObject::from_bytes(json.into_bytes()))
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

macro_rules! rpc_call {
    ($call:ident, $sel: ident) => ({
        match JsonBuilder::default()
            .method(ApiCall::$call).build().map_err(|e| futures::future::err(e.into())) {
                Ok(j) => Box::new($sel.do_post(j)),
                Err(e) => Box::new(e)
            }
    })
}

impl EthRpcClient for InfuraClient {
    fn getBlockNumber(&self) -> Box<dyn Future<Item=ResponseObject, Error = Error>> {
        rpc_call!(EthBlockNumber, self)
    }

    fn getBlockByNumber(&self) -> Box<Future<Item=ResponseObject, Error = Error>> {
        rpc_call!(EthGetBlockByNumber, self)
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
    use regex::Regex;
 
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
}
