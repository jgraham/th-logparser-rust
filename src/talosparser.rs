use logparser::LogParser;
use regex::Regex;
use rustc_serialize::json::{self, Json};
use std::mem;

lazy_static! {
    static ref RE_TALOS: Regex =
        Regex::new(r"TALOSDATA:\s+(?P<data>\[.*\])").unwrap();
}

pub struct TalosParser {
    artifact: Option<Json>,
    complete: bool
}

impl TalosParser {
    pub fn new() -> TalosParser {
        TalosParser {
            artifact: None,
            complete: false
        }
    }
}

impl LogParser for TalosParser {
    fn name(&self) -> &'static str {
        "talos_data"
    }
    
    fn parse_line(&mut self, line: &str, _line_number: u32) {
        if RE_TALOS.is_match(line) {
            let matches = RE_TALOS.captures(line).unwrap();
            let json_data = matches.name("data").unwrap_or("{}");
            self.artifact = Some(Json::from_str(json_data).unwrap());
            self.complete = true;
        }
    }

    fn get_artifact(&mut self) -> String {
        let artifact = if self.artifact.is_some() {
            mem::replace(&mut self.artifact, None).unwrap()
        } else {
            Json::Array(vec![])
        };
        json::encode(&artifact).unwrap()                
    }

    fn has_artifact(&self) -> bool {
        self.artifact.is_some()
    }

    fn complete(&self) -> bool {
        self.complete
    }
}
