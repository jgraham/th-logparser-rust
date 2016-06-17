extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate regex;
extern crate rustc_serialize;
extern crate time;
extern crate hyper;
extern crate flate2;

pub mod http;
pub mod logparser;
pub mod performanceparser;
pub mod stepparser;
pub mod tinderboxparser;

use http::{get, read};
use libc::c_char;
use logparser::{LogParser, LogParserError};
use std::error::Error;
use std::ffi::{CStr, CString};
use std::io::BufRead;
use std::str;
use std::time::Duration;

pub fn parse_log(url: &str, user_agent: &str) -> Result<Vec<(&'static str, String)>, LogParserError> {
    let mut rv = Vec::with_capacity(3);

    let mut step_parser = stepparser::StepParser::new();
    let mut tinderbox_parser = tinderboxparser::TinderboxParser::new();
    let mut performance_parser = performanceparser::PerformanceParser::new();

    let mut final_line_number = 0;
    let http_resp = try!(get(url, user_agent, Some(Duration::new(30, 0))));
    for (line_number, maybe_line) in try!(read(http_resp)).lines().enumerate() {
        if let Ok(line) = maybe_line {
            try!(parse_line(&mut step_parser, &*line, line_number as u32));
            try!(parse_line(&mut tinderbox_parser, &*line, line_number as u32));
            try!(parse_line(&mut performance_parser, &*line, line_number as u32));
            final_line_number = line_number as u32;
        }
    }

    finish_parse(&mut step_parser, final_line_number, &mut rv);
    finish_parse(&mut tinderbox_parser, final_line_number, &mut rv);
    finish_parse(&mut performance_parser, final_line_number, &mut rv);
    Ok(rv)
}

fn parse_line<T: LogParser>(parser: &mut T, line: &str, line_number: u32) -> Result<(), LogParserError> {
    if !parser.complete() {
        try!(parser.parse_line(line, line_number as u32));
    };
    Ok(())
}


fn finish_parse<T: LogParser>(parser: &mut T, final_line_number: u32,
                              rv: &mut Vec<(&'static str, String)>) {
    parser.finish_parse(final_line_number);
    if parser.has_artifact() {
        rv.push((parser.name(), parser.get_artifact()));
    }
}

#[no_mangle]
pub extern fn parse_artifacts(url_cstr: *const c_char, ua_cstr: *const c_char) -> *const c_char {
    let url = unsafe {
        if url_cstr.is_null() {
            return CString::new("").unwrap().into_raw();
        }
        str::from_utf8(CStr::from_ptr(url_cstr).to_bytes()).unwrap()
    };
    let user_agent = unsafe {
        if ua_cstr.is_null() {
            return CString::new("").unwrap().into_raw();
        }
        str::from_utf8(CStr::from_ptr(ua_cstr).to_bytes()).unwrap()
    };
    let resp = match parse_log(url, user_agent) {
        Ok(items) => {
            let item_len: usize = items.iter().map(|x| x.0.len() + x.1.len() + url.len() + 23).fold(0, |acc, x| acc + x);
            let mut buf = String::with_capacity(item_len + 3);
            buf.push_str("0\x17");
            for &(ref parser_name, ref artifact) in items.iter() {
                buf.push_str(&*format!("{{\"{}\": {}, \"logurl\": \"{}\"}}\x17",
                                       parser_name, artifact, url))
            };
            buf
        },
        Err(e) => {
            format!("1\x17{}\x17{}", e.name(), e.description())
        }
    };
    CString::new(resp.into_bytes()).unwrap().into_raw()
}
