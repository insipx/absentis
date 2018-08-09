#![feature(rust_2018_preview, fs_read_write, use_extern_macros)]

#[macro_use] mod utils;
mod node;
mod config_file;
mod err;
mod client;
mod types;
mod transaction_finder;

// TODO SOMETIME BEFORE RELEASE
//  - make errors nice and not sloppy

fn main() {


    println!("Hello, world!");
}
