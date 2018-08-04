#[cfg(test)]
use super::*;
use log::*;
use colored::Colorize;
use regex::Regex;
use std::sync::{Once, ONCE_INIT};
use env_logger;

#[test]
fn it_should_get_the_latest_block() {
    env_logger::try_init();
    //pub fn get_latest_block(conf: Configuration) -> Result<(), Error>  {
    let client = InfuraClient::new().expect("Error building client!");
    
    let task = client.getBlockNumber().map_err(|err: failure::Error| { 
        pretty_err!(err);
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
        pretty_err!(err);
        panic!("Failed due to error");
    }).and_then(|res| {
        info!("{}: {:?}","eth_getBlockByNumber".green().bold(), res);
        assert_eq!(res.to_str(), "EthGetBlockByNumber");
        Ok(())
    });
    let mut rt = tokio::runtime::Runtime::new().expect("Could not construct tokio runtime");
    rt.block_on(task);
}

