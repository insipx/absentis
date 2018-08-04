#![feature(rust_2018_preview, fs_read_write, use_extern_macros)]
#[macro_use] mod utils;
mod node;
mod ethereum_objects;
mod conf;
mod err;
mod types;
mod json_builder;
mod rpc_client;

// TODO SOMETIME BEFORE RELEASE
//  - make errors nice and not sloppy

fn main() {


    println!("Hello, world!");
}
