#![feature(rust_2018_preview, fs_read_write, use_extern_macros)]
use log::*;
#[macro_use] mod utils;
mod config_file;
mod conf;
mod cli;
mod node;
mod err;
mod client;
mod types;
mod transaction_finder;

// TODO SOMETIME BEFORE RELEASE
//  - make errors nice and not sloppy

fn main() {
    pretty_env_logger::init();
    let _conf = match conf::Configuration::new() {
        Err(e) => {
            error!("{}", e);
            trace!("Cause: {}", e.as_fail());
            trace!("Backtrace: {:#?}", e.backtrace());
            std::process::exit(1);
        },
        Ok(v) => v
    };
}
