extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate time;
extern crate rustc_serialize;

mod stepparser;
mod logparser;

use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use time::now;
use stepparser::StepParser;
use logparser::LogParser;

fn main() {
    let f = File::open("log.txt").expect("Couldn't open log.txt");
    let mut data = String::with_capacity(f.metadata().unwrap().len() as usize);
    let mut reader = BufReader::new(f);
    reader.read_to_string(&mut data).unwrap();
    let mut parser = StepParser::new();

    let start = now();
    let mut line_number = 0 as u32;
    for line in data.lines() {
        parser.parse_line(line, line_number);
        line_number += 1;
    }
    parser.finish_parse(line_number - 1);
    println!("{}", parser.get_artifact());
    let duration = now() - start;
    println!("{}", f64::from(duration.num_milliseconds() as u32) / 1000.);
}

