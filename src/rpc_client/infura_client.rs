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
use crate::utils::IntoHexStr;
use super::response_object::ResponseObject;
use super::jsonrpc_object::JsonRpcObject;
use super::err::RpcError;
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
    ($call:ident, $sel: ident, $params: expr) => ({
        match JsonRpcObject::default()
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


#[cfg(test)]
mod tests {
    use regex::Regex;
    use crate::types::Uri;
    use super::*;

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

