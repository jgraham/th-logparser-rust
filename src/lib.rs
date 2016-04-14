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

use libc::c_char;
use std::ffi::{CStr, CString};
use std::str;
use logparser::LogParser;
use std::time::Duration;
use http::{get, read};
use std::io::BufRead;

pub fn parse_log(url: &str, user_agent: &str) -> Vec<String> {
    let mut rv = Vec::with_capacity(4);

    let mut step_parser = stepparser::StepParser::new();
    let mut tinderbox_parser = tinderboxparser::TinderboxParser::new();
    let mut performance_parser = performanceparser::PerformanceParser::new();

    let mut final_line_number = 0;
    for (line_number, maybe_line) in read(get(url, user_agent, Some(Duration::new(30, 0))).unwrap()).lines().enumerate() {
        if let Ok(line) = maybe_line {
            parse_line(&mut step_parser, &*line, line_number as u32);
            parse_line(&mut tinderbox_parser, &*line, line_number as u32);
            parse_line(&mut performance_parser, &*line, line_number as u32);
            final_line_number = line_number as u32;
        }
    }

    finish_parse(&mut step_parser, final_line_number, &*url, &mut rv);
    finish_parse(&mut tinderbox_parser, final_line_number, &*url, &mut rv);
    finish_parse(&mut performance_parser, final_line_number, &*url, &mut rv);
    rv
}

fn parse_line<T: LogParser>(parser: &mut T, line: &str, line_number: u32) {
    if !parser.complete() {
        parser.parse_line(line, line_number as u32);
    }
}


fn finish_parse<T: LogParser>(parser: &mut T, final_line_number: u32, url: &str,
                              rv: &mut Vec<String>) {
    parser.finish_parse(final_line_number);
    if parser.has_artifact() {
        rv.push(
            format!("{{\"{}\": {}, \"logurl\": \"{}\"}}",
                    parser.name(), parser.get_artifact(), url));
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
    let items = parse_log(url, user_agent);
    CString::new(items.join("\x17").into_bytes()).unwrap().into_raw()
}
