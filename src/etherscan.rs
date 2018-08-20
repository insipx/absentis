use log::*;
#[macro_use] mod types;
pub use self::types::{EtherScanTx, EtherScanResponse};
use hyper::client::HttpConnector;
use futures::{
    future::Future,
    stream::Stream,
};
use failure::Fail;
use web3::types::{H160};

pub struct EtherScan {
    client: hyper::client::Client<HttpConnector, hyper::Body>,
}

pub enum SortType {
    Ascending,
    Descending,
    None
}

impl From<SortType> for String {
    fn from(sort_type: SortType) -> String {
        match sort_type {
            SortType::Ascending => "asc".to_string(),
            SortType::Descending => "des".to_string(),
            SortType::None => "asc".to_string()
        }
    }
}

impl EtherScan {
    pub fn new() -> Self {
        EtherScan {
            client: hyper::client::Client::new()
        }
    }

    pub fn get_tx_by_account(&self, ev_loop: &mut tokio_core::reactor::Core,
                             addr: H160,
                             from: u64,
                             to: u64,
                             sort: SortType)
                             -> Result<Vec<EtherScanTx>, EtherScanError>
    {
        let req = eth_txlist!(addr, from.to_string(), to.to_string(), String::from(sort));
        info!("URL: {}", req);
        let res =  ev_loop.run(self.do_get(req.parse().expect("URI should not be invalid")))?;
        let response = serde_json::from_slice::<EtherScanResponse<Vec<EtherScanTx>>>(&res.to_vec())?;
        Ok(response.result)
    }

    fn do_get(&self, uri: hyper::Uri) -> impl Future<Item = bytes::Bytes, Error = EtherScanError> {
        self.client
            .get(uri)
            .and_then(|res| {
                assert_eq!(res.status(), hyper::StatusCode::OK);
                res.into_body().concat2()
            })
            .map_err(|e| e.into())
            .and_then(|json| {
                futures::future::result(Ok(json.into_bytes()))
            })
    }
}


#[derive(Fail, Debug)]
pub enum EtherScanError {
    #[fail(display = "Could not decode Etherscan Response: {}", _0)]
    FailedToDecode(#[cause] serde_json::Error),
    #[fail(display = "Failed to make request: {}", _0)]
    RequestError(#[cause] hyper::error::Error),
    #[fail(display = "Conversion error; {}", _0)]
    ConversionError(#[cause] std::str::Utf8Error)
}

impl From<serde_json::Error> for EtherScanError {
    fn from(err: serde_json::Error) -> EtherScanError {
        EtherScanError::FailedToDecode(err)
    }
}

impl From<hyper::error::Error> for EtherScanError {
    fn from(err: hyper::error::Error) -> EtherScanError {
        EtherScanError::RequestError(err)
    }
}

impl From<std::str::Utf8Error> for EtherScanError {
    fn from(err: std::str::Utf8Error) -> EtherScanError {
        EtherScanError::ConversionError(err)
    }
}
