use log::{log, error};
use std::path::PathBuf;
use web3::BatchTransport;
use web3::transports;
// use tokio::reactor::Reactor;
use failure::Error;
use super::types::MAX_PARALLEL_REQUESTS;
use super::conf::Configuration;
use super::err::ClientError;

pub struct Client<T: BatchTransport> {  
    web3: web3::Web3<T>,
    ev_loop: tokio_core::reactor::Core,
}

impl<T> Client<T> where T: BatchTransport {
    pub fn new(transport: T) -> Result<Self, Error> {
        let ev_loop = tokio_core::reactor::Core::new()?; 
        Ok(Client {
            web3: web3::Web3::new(transport),
            ev_loop,
        })
    }

    pub fn new_ipc(conf: &Configuration) -> Result<Client<transports::ipc::Ipc>, Error> {
        let ev_loop = tokio_core::reactor::Core::new()?; 
        ClientBuilder::ipc()
            .path(conf.ipc_path()?)
            .handle(ev_loop.handle())
            .build(ev_loop)
            .map_err(|e| e.into())
    }

    pub fn new_http(conf: &Configuration) -> Result<Client<transports::http::Http>, Error> {
        let ev_loop = tokio_core::reactor::Core::new()?; 
        ClientBuilder::http()
           .url(conf.url()?)
           .handle(ev_loop.handle())
           .build(ev_loop)
           .map_err(|e| e.into())
    }
}

struct HttpBuilder {
    url: Option<String>,
    max_parallel: Option<usize>,
    handle: Option<tokio_core::reactor::Handle>,
}

impl HttpBuilder {
    fn url(&mut self, val: String) -> &mut Self {
        let new = self;
        new.url = Some(val);
        new
    }
    fn max_parallel(&mut self, val: usize) -> &mut Self {
        let new = self;
        new.max_parallel = Some(val);
        new
    }
    fn handle(&mut self, val: tokio_core::reactor::Handle) -> &mut Self {
        let new = self;
        new.handle = Some(val);
        new
    }

    fn build(&self, ev_loop: tokio_core::reactor::Core) ->  Result<Client<transports::http::Http>, ClientError> {
        let url = self.url.as_ref().ok_or_else(|| ClientError::MustSpecify("URL".into()))?;
        let handle = self.handle.as_ref().ok_or_else(|| ClientError::MustSpecify("URL".into()))?;
        let max = self.max_parallel.unwrap_or(MAX_PARALLEL_REQUESTS);
        let http = 
            web3::transports::Http::with_event_loop(url,handle,max);

        let http = match http {
            Ok(v) => v,
            Err(e) => {
                pretty_err!("{}{}", "Could not initialize the Web3 Object: ", e.description());
                if let Some(bt) = e.backtrace() {
                    // pretty_err!("{}{}", "Backtrace: ", bt);
                    println!("Backtrace: {:?}", bt);
                }
                panic!("Shutting down...");
            }
        };


        Ok(Client {
            web3: web3::Web3::new(http),
            ev_loop,
        })
    }
}

struct IpcBuilder {
    path: Option<std::path::PathBuf>,
    handle: Option<tokio_core::reactor::Handle>,
}

impl IpcBuilder {
    fn path(&mut self, path: PathBuf) -> &mut Self {
        let new = self;
        new.path = Some(path);
        new
    }
    fn handle(&mut self, handle: tokio_core::reactor::Handle) -> &mut Self {
        let new = self;
        new.handle = Some(handle);
        new
    }
    fn build(&self, ev_loop: tokio_core::reactor::Core) -> Result<Client<transports::ipc::Ipc>, ClientError> {
        let path = self.path.as_ref().ok_or_else(||ClientError::MustSpecify("Path".into()))?;
        let handle = self.handle.as_ref().ok_or_else(||ClientError::MustSpecify("Tokio-core Handle".into()))?;
        let ipc = web3::transports::Ipc::with_event_loop(path.as_path(), handle);

        let ipc = match ipc {
            Ok(v) => v,
            Err(e) => {
                pretty_err!("{}{}", "Could not initialize the Web3 Object: ", e.description());
                if let Some(bt) = e.backtrace() {
                    // pretty_err!("{}{}", "Backtrace: ", bt);
                    println!("Backtrace: {:?}", bt);
                }
                panic!("Shutting down...");
            }
        };

        Ok(Client {
            web3: web3::Web3::new(ipc),
            ev_loop,
        })
    }
}

#[derive(Default)]
pub struct ClientBuilder;


impl ClientBuilder {
    fn ipc() -> IpcBuilder {
        IpcBuilder {
            path: None,
            handle: None
        }
    }

    fn http() -> HttpBuilder {
        HttpBuilder {
            url: None,
            max_parallel: None,
            handle: None,
        }
    }
}
