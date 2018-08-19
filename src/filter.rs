use log::*;
use web3::{
    BatchTransport,
    types::{Trace, H160, H256, Index, BlockNumber, TraceFilterBuilder},
};
use futures::future::Future;
use super::{
    utils,
    client::Client,
};

// an attempted implementation of
// https://medium.com/@tjayrush/defeating-the-ethereum-ddos-attacks-d3d773a9a063
pub struct TraceFilterCall;
impl TraceFilterCall {

    pub fn is_spam<T>(client: &mut Client<T>,hash: H256, block: u64) -> bool
    where
        T: BatchTransport
    {
        if block < 2286910 || block > 2463000 {
            false
        } else  {
            Self::get_trace_count(client, hash, 0, 1000000) > 250
        }
    }

    // Code Progress: Prototype, TODO: handle errors #p2
    fn get_trace_count<T>(client: &mut Client<T>, hash: H256, first: usize, last:usize) -> usize
    where
        T: BatchTransport
    {
        if last == first {
            return first;
        }

        let mid: usize = first + ((last - first) / 2);
        let has_trace:bool = Self::has_trace_at(client, hash, mid.into()).expect(&verb_msg!("Should not fail"));

        if has_trace && !Self::has_trace_at(client, hash, (mid + 1).into()).expect(&verb_msg!("Should not fail")) {
            return mid;
        }
        if !has_trace {
            return Self::get_trace_count(client, hash, first, mid-1);
        }
        return Self::get_trace_count(client, hash, mid+1, last);
    }

    fn has_trace_at<T>(client: &mut Client<T>, hash: H256, index: Index) -> Result<bool, web3::error::Error>
    where
        T: BatchTransport
    {
        let fut = client.web3.trace().get(hash, vec![index]);
        let trace = client.run(fut)?;
        Ok(trace.block_number >= 0)
    }
}

// Request transactions asynchronously
/// Request transactions asynchronously. If a pending blocknumber is specified, defaults to `latest`
pub fn trace_filter<T>(client: &Client<T>, to_block: Option<BlockNumber>, to_address: H160)
    -> impl Future<Item = Vec<Trace>, Error = ()>
    where
        T: BatchTransport,
{
    let to_block = to_block.unwrap_or(BlockNumber::Latest);
    let to_block: u64 = match to_block {
        BlockNumber::Earliest => 0,
        BlockNumber::Latest => utils::latest_block(client),
        BlockNumber::Number(num) => num,
        BlockNumber::Pending => utils::latest_block(client),
    };

    let filter = |from, to, addr| {
        let trace_filter = TraceFilterBuilder::default()
            .from_block(BlockNumber::Number(from))
            .to_block(BlockNumber::Number(to))
            .to_address(vec![addr])
            .build();

        client.web3_batch.trace()
            .filter(trace_filter);
    };

    // TODO: possibly may skip over some transactions
    let chunk = to_block / 4;
    let mut to_chunk = to_block / 4;
    let mut last_block = 0;
    while to_chunk <= to_block {
        info!("Querying batch: {} to: {}", last_block, to_chunk);
        filter(last_block,to_chunk,to_address.clone());
        last_block = to_chunk;
        to_chunk = last_block + chunk;
    }

    client.web3_batch.transport().submit_batch().then(|x| {
        let x = try_web3!(x);
        let new_val: Vec<Trace> = x.into_iter().flat_map(|batch| {
            let batch = try_web3!(batch);
            serde_json::from_value::<Vec<Trace>>(batch).expect("Deserialization should never fail")
        }).collect();
        futures::future::ok(new_val)
    }).map_err(|e: web3::Error|{
        panic_web3!(e);
    })
}
