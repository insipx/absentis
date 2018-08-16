#![feature(rust_2018_preview, fs_read_write)]
#[macro_use] mod utils;
mod types;
mod config_file;
mod conf;
mod cli;
mod node;
mod err;
mod client;
mod transaction_finder;
mod transaction_validator;
mod filter;
mod etherscan;

use log::*;
use failure::Error;

// TODO SOMETIME BEFORE RELEASE
//  - make errors nice and not sloppy

fn main() -> Result<(), Error>{
    pretty_env_logger::init();
    match conf::Configuration::new() {
        Err(e) => {
            error!("{}", e);
            trace!("Cause: {}", e.as_fail());
            trace!("Backtrace: {:#?}", e.backtrace());
            Err(e)
        },
        Ok(v) => Ok(())
    }
}
