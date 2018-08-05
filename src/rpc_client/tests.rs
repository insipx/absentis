#[cfg(test)]
use super::*;
use log::*;
use colored::Colorize;
use regex::Regex;
use std::sync::{Once, ONCE_INIT};
use std::str::FromStr;
use crate::ethereum_objects::Hex;
use env_logger;


#[test]
fn it_should_get_the_latest_block() {
    env_logger::try_init();
    //pub fn get_latest_block(conf: Configuration) -> Result<(), Error>  {
    let client = InfuraClient::new().expect("Error building client!");
    
    let task = client.block_number().map_err(|err: failure::Error| { 
        pretty_err!(err);
        panic!("Failed due to error");
    }).and_then(|res| {
        pretty_success!("eth_blockNumber", "{:?}", res);
//        compare!(EthBlockNumber, "EthBlockNumber", res);
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
    
    let task = client.get_block_by_number(300, true).map_err(|err: failure::Error| { 
        pretty_err!(err);
        panic!("Failed due to error");
    }).and_then(|res| {
        pretty_success!("eth_getBlockByNumber", "{:?}", res);
        // compare!(EthGetBlockByNumber, "EthGetBlockByNumber", res);
        Ok(())
    });
    let mut rt = tokio::runtime::Runtime::new().expect("Could not construct tokio runtime");
    rt.block_on(task);
}

#[test]
fn it_should_get_gas_price() {
    env_logger::try_init();
    let client = InfuraClient::new().expect("Error building client!");

    let task = client.gas_price().map_err(|err: failure::Error| {
        pretty_err!(err);
        panic!("Failed due to err");
    }).and_then(|res| {
        pretty_success!("eth_gasPrice", "{:?}", res);
        // compare!(EthGasPrice, "EthGasPrice", res);
        Ok(())
    });


    let mut rt = tokio::runtime::Runtime::new().expect("Could not construct tokio runtime");
    rt.block_on(task);
}

#[test]
fn it_should_get_balance() {
    env_logger::try_init();
    let client = InfuraClient::new().expect("Error building client!");
    
    // either of these works: 
    // let addr: Address = serde_json::from_str("\"0x884531EaB1bA4a81E9445c2d7B64E29c2F14587C\"").expect("Couldn't deserialize addr");
    let addr: Address = Address::from_str("884531EaB1bA4a81E9445c2d7B64E29c2F14587C").expect("String -> Address parse error");
    println!("ADDR: {}", addr.to_string().red().bold().underline().blink());
    let task = client.get_balance(addr, None, Some(BlockString::Latest)).map_err(|err: failure::Error| {
        pretty_err!(err);
        panic!("Failed due to err");
    }).and_then(|res| {
        pretty_success!("eth_getBalance", "{:?}", res);
        // compare!(EthGetBalance, "EthGetBalance", res);
        Ok(())
    });


    let mut rt = tokio::runtime::Runtime::new().expect("Could not construct tokio runtime");
    rt.block_on(task);
}

