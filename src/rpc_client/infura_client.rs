use failure::*;
use futures::{Future};
use hyper::{Client, Uri as HyperUri, Method, Request};
use hyper::rt::{Stream};
use hyper_tls::HttpsConnector;
use hyper::client::{HttpConnector, ResponseFuture};
use hyper::header::HeaderValue;
use ethereum_types::{Address, H256};
use serde_json as ser;

use crate::ethereum_objects::{BlockString, EthObjType, Block, Hex, Transaction};
use crate::types::*;
use crate::conf::Configuration;
use crate::utils::IntoHexStr;

use super::request_object::RequestObject;
use super::response_object::ResponseObject;
use super::err::{RpcError, ResponseBuildError, TypeMismatchError};
use super::EthRpcClient;
use super::api_call::ApiCall;

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
    ($obj:ident, $sel:ident, $call: ident, $params: expr) => ({
        let j = try_future!(
             RequestObject::default()
                .method(ApiCall::$call)
                .params($params.to_vec())
                .build()
        );
        
        let res = $sel.do_post(j)
            .and_then(|resp| {
                let res = match_response!(resp);
                res.map_err(|e| e.into())
            }).and_then(|res| {
                if let EthObjType::$obj(obj) = res {
                    futures::future::ok(obj)
                } else {
                    futures::future::err(ResponseBuildError::MismatchedTypes(TypeMismatchError::new(stringify!($obj).into(), res.into())).into())
                }
            });
        Box::new(res)
    })
}

macro_rules! de_addr {
    ($addr:ident) => ({
        ser::Value::String(format!("{:?}", $addr))
    })
}

macro_rules! de_hash {
    ($hash:ident) => ({
        ser::Value::String(format!("{:?}", $hash))
    })
}

macro_rules! de_str {
    ($str:expr) => ({
        ser::Value::String($str)
    })
}

macro_rules! de_bool {
    ($bool:expr) => ({
        ser::Value::Bool($bool)
    })
}

impl EthRpcClient for InfuraClient {
    
    fn block_number(&self) -> Box<dyn Future<Item=Hex, Error = Error> + Send> {
        return rpc_call!(Hex, self, EthBlockNumber, []);
    }

    fn gas_price(&self) -> Box<dyn Future<Item=Hex, Error=Error> + Send> {
        return rpc_call!(Hex, self, EthGasPrice, []);
    }

    fn get_balance(&self, addr: Address, block_num: Option<usize>, block_str: Option<BlockString>
        ) -> Box<dyn Future<Item=Hex, Error=Error> + Send>
    {
        if block_num.is_some() {
            return rpc_call!(Hex, self, EthGetBalance, [de_addr!(addr), de_str!(block_num.expect("scope is conditional; qed").into_hex_str())]);
        } else if block_str.is_some() {
            return rpc_call!(Hex, self, EthGetBalance, [de_addr!(addr), de_str!(block_str.expect("scope is conditional; qed").to_str())]);
        } else {
            return Box::new(futures::future::err(RpcError::MissingParameter("Missing `block_num` or `block_str`".to_owned()).into()));
        }
    }
   
    fn get_block_by_hash(&self, hash: H256) 
        -> Box<Future<Item=Block, Error=Error> + Send> 
    {   
        return rpc_call!(Block, self, EthGetBlockByHash, [de_hash!(hash), de_bool!(true)]);
    }
   
    fn get_block_by_number(&self, block_num: u64) 
        -> Box<Future<Item=Block, Error = Error> + Send>
    {
        return rpc_call!(Block, self, EthGetBlockByNumber, [de_str!(block_num.into_hex_str()), de_bool!(true)]);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use log::*;
    use colored::Colorize;
    use regex::Regex;
    use crate::types::Uri;

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

