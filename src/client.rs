use log::{log, error};
use std::path::PathBuf;
use web3::BatchTransport;
use web3::transports;
// use tokio::reactor::Reactor;
use failure::Error;
use super::types::MAX_PARALLEL_REQUESTS;
use super::conf::{Configuration, TransportType};
use super::err::ClientError;

pub struct Client<T: BatchTransport> {  
    web3: web3::Web3<T>,
    ev_loop: tokio_core::reactor::Core,
}

impl<T> Client<T> where T: BatchTransport {
    pub fn new(conf: &Configuration) -> Result<Self, Error> {
        let event_loop = tokio_core::reactor::Core::new()?;
        
        
        if let TransportType::Http = conf.transport() {
            ClientBuilder::http()
                .url(conf.url()?)
                .handle(event_loop.handle())
                .build(event_loop).map_err(|e| e.into())
        } else if let TransportType::Ipc = conf.transport() {
            ClientBuilder::ipc()
                .path(conf.ipc_path()?)
                .handle(event_loop.handle())
                .build(event_loop).map_err(|e| e.into())
        } else {
            return Err(ClientError::MustSpecify("A Transport Type".into()).into())
        }
    }
}

struct HttpBuilder {
    url: Option<String>,
    max_parallel: Option<usize>,
    handle: Option<tokio_core::reactor::Handle>,
}

impl HttpBuilder {
    fn url(&mut self, val: String) -> &mut Self {
        let mut new = self;
        new.url = Some(val);
        new
    }
    fn max_parallel(&mut self, val: usize) -> &mut Self {
        let mut new = self;
        new.max_parallel = Some(val);
        new
    }
    fn handle(&mut self, val: tokio_core::reactor::Handle) -> &mut Self {
        let mut new = self;
        new.handle = Some(val);
        new
    }

    fn build(&self, ev_loop: tokio_core::reactor::Core) ->  Result<Client<transports::http::Http>, ClientError> {
        let url = self.url.ok_or_else(|| ClientError::MustSpecify("URL".into()))?;
        let handle = self.handle.ok_or_else(|| ClientError::MustSpecify("URL".into()))?;
        let max = self.max_parallel.unwrap_or(MAX_PARALLEL_REQUESTS);
        let http = 
            web3::transports::Http::with_event_loop(&url,&handle,max);

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
        let mut new = self;
        new.path = Some(path);
        new
    }
    fn handle(&mut self, handle: tokio_core::reactor::Handle) -> &mut Self {
        let mut new = self;
        new.handle = Some(handle);
        new
    }
    fn build(&self, ev_loop: tokio_core::reactor::Core) -> Result<Client<transports::ipc::Ipc>, ClientError> {
        let path = self.path.ok_or(ClientError::MustSpecify("Path".into()))?;
        let handle = self.handle.ok_or(ClientError::MustSpecify("Tokio-core Handle".into()))?;
        let ipc = web3::transports::Ipc::with_event_loop(path.as_path(), &handle);

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
