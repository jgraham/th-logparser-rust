extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate time;
extern crate rustc_serialize;
extern crate hyper;
extern crate flate2;
extern crate logparser;

use logparser::parse_log;
use std::error::Error;

fn main() {
    let url =
        "http://archive.mozilla.org/pub/firefox/tinderbox-builds/mozilla-inbound-linux64-st-an-debug/1460667041/mozilla-inbound-linux64-st-an-debug-bm74-build1-build694.txt.gz";
    let user_agent = "Log Parser Test";
    match parse_log(url, user_agent) {
        Ok(artifacts) => {
            println!("Got {} artifacts", artifacts.len());
            for &(ref parser_name, ref artifact) in artifacts.iter() {
                println!("{} {}", parser_name, artifact);
            }
        }
        Err(e) => {
            println!("Got error {}", e.description());
        }
    }
}
