use ethereum_types::Address;
use web3::futures::Future;
use tokio_core::reactor;


fn query_transactions_at() {
    let mut event_loop = reactor::Core::new().unwrap();
    let handle = event_loop.handle();
    let ipc = web3::transports::Ipc::with_event_loop("",&handle).unwrap();
    let web3 = web3::Web3::new(ipc);
}
