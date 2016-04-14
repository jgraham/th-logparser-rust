use logparser::LogParser;
use regex::Regex;
use rustc_serialize::json;
use std::mem;

lazy_static! {
    static ref RE_PERFORMANCE: Regex =
        Regex::new(r"PERFHERDER_DATA:\s+(?P<data>\{.*\})").unwrap();
}

pub struct PerformanceParser {
    artifact: Vec<String>,
}

impl PerformanceParser {
    pub fn new() -> PerformanceParser {
        PerformanceParser {
            artifact: vec![],
        }
    }
}

impl LogParser for PerformanceParser {
    fn name(&self) -> &'static str {
        "performance_data"
    }
    
    fn parse_line(&mut self, line: &str, _line_number: u32) {
        if RE_PERFORMANCE.is_match(line) {
            let matches = RE_PERFORMANCE.captures(line).unwrap();
            let json_data = matches.name("data").unwrap_or("{}");
            // The rust JSON parser is spec-compliant, but the Python one is not and allows
            // NaN as a number. To work around this just push the string here and reparse
            // the result on the python side
            self.artifact.push(json_data.into());
        }
    }

    fn has_artifact(&self) -> bool {
        self.artifact.len() > 0
    }

    fn get_artifact(&mut self) -> String {
        let artifact = mem::replace(&mut self.artifact, vec![]);
        json::encode(&artifact).unwrap()
    }
}
