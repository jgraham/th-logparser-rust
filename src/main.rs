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

fn main() {
    let url =
        "http://archive.mozilla.org/pub/firefox/tinderbox-builds/mozilla-inbound-linux64-st-an-debug/1460667041/mozilla-inbound-linux64-st-an-debug-bm74-build1-build694.txt.gz";
    let user_agent = "Log Parser Test";
    let artifacts = parse_log(url, user_agent);
    println!("Got {} artifacts", artifacts.len());
    for artifact in artifacts.iter() {
        println!("{}", artifact);
    }

}
