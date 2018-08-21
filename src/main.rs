#![feature(try_from, transpose_result)]
#[macro_use] mod utils;
mod types;
mod config_file;
mod conf;
mod cli;
// mod node;
mod err;
mod client;
// mod transaction_finder;
mod transaction_validator;
// mod filter;
mod etherscan;
use failure::Error;

// TODO SOMETIME BEFORE RELEASE
//  - make errors nice and not sloppy

fn main() -> Result<(), Error>{
    pretty_env_logger::init();
    let conf = conf::Configuration::new()?;
    Ok(())
}
