use web3::Transport;
use web3::transports::EventLoopHandle;
use failure::Error;
use super::conf::{Configuration, TransportType};

pub struct Client<T> where T: Transport {  
    web3: web3::Web3<T>,
    ev_loop: EventLoopHandle,
}

impl<T> Client<T> where T: Transport {
    pub fn new(conf: &Configuration) -> Result<Self, Error> {
        if let TransportType::Http = conf.transport() {
            let (_eloop, transport) = web3::transports::Http::new(&conf.url()?)?;
            Ok(())
        } else if let TransportType::Ipc = conf.transport() {
            let (_eloop, transport) = web3::transports::Ipc::new(&conf.ipc_path()?)?;
            Ok(())
        } else {
            return Err(ClientError::MustSpecify("A Transport Type".into()))
        }
    }
}
