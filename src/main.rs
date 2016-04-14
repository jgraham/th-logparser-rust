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
        "https://public-artifacts.taskcluster.net/SQcZMzk4SYm_ppI6rXhWcQ/0/public/logs/live_backing.log";
    let user_agent = "Log Parser Test"; let artifacts = parse_log(url,
    user_agent); for artifact in artifacts.iter() { println!("{}",
    artifact); }

}
