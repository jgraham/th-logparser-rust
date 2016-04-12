use chrono::{UTC, TimeZone};
use regex::{Regex, RegexSet};
use rustc_serialize::{Encodable, Encoder};
use rustc_serialize::json::{self, Json, ToJson};
use std::convert::From;
use std::mem;
use logparser::LogParser;

static PARSER_MAX_STEP_ERROR_LINES: u8 = 100;

lazy_static! {
    static ref RE_HEADER_LINE: Regex =
        Regex::new("^(?:builder|slave|starttime|results|buildid|builduid|revision): ").unwrap();

    static ref RE_STEP_MARKER: Regex =
        Regex::new(
r#"={9} (?P<marker_type>Started|Finished) (?P<name>.*?) \(results: (?P<result_code>\d+), elapsed: .*?\) \(at (?P<timestamp>.*?)\)"#).unwrap();

    static ref RE_ERRORLINE_SET_1: RegexSet = {
        RegexSet::new(&[
            r"TEST-(?:INFO|PASS) ", // Line to always exclude
            r"^\d+:\d+:\d+ +(?:ERROR|CRITICAL|FATAL) - " // Line to always include
        ]).unwrap()
    };

    static ref RE_MOZHARNESS_PREFIX: Regex =
        Regex::new(r"^\d+:\d+:\d+ +(?:DEBUG|INFO|WARNING) - +").unwrap();

    static ref RE_ERROR_EXCLUDE: RegexSet =
        RegexSet::new(&[
            r"I[ /](Gecko|Robocop|TestRunner).*TEST-UNEXPECTED-",
            r"^TimeoutException: ",
            r"^ImportError: No module named pygtk$"]
        ).unwrap();

    static ref RE_ERROR_TERMS: RegexSet = RegexSet::new(&[
        r"TEST-UNEXPECTED-",
        r"fatal error",
        r"FATAL ERROR",
        r"PROCESS-CRASH",
        r"Assertion failure:",
        r"Assertion failed:",
        r"###!!! ABORT:",
        r"E/GeckoLinker",
        r"SUMMARY: AddressSanitizer",
        r"SUMMARY: LeakSanitizer",
        r"Automation Error:",
        r"command timed out:",
        r"wget: unable ",
        r"TEST-VALGRIND-ERROR",
        r"^error: TEST FAILED",
        r"^g?make(?:\[\d+\])?: \*\*\*",
        r"^Remote Device Error:",
        r"^[A-Za-z.]+Error: ",
        r"^[A-Za-z.]*Exception: ",
        r"^remoteFailed:",
        r"^rm: cannot ",
        r"^abort:",
        r"^Output exceeded \d+ bytes",
        r"^The web-page 'stop build' button was pressed",
        r".*\.js: line \d+, col \d+, Error -",
        r"^\[taskcluster\] Error:",
        r"^\[[\w-]+:(?:error|exception)\]",
        r" error\(\d*\):",
        r":\d+: error:",
        r" error R?C\d*:",
        r"ERROR [45]\d\d:",
        r"mozmake\.exe(?:\[\d+\])?: \*\*\*"]).unwrap();
}

#[derive(Debug)]
enum StepResult {
    Unknown,
    Success,
    TestFailed,
    Busted,
    Skipped,
    Exception,
    Retry,
    UserCancel,
}

impl StepResult {
    fn from_str(data: &str) -> StepResult {
        match data {
            "0" => StepResult::Success,
            "1" => StepResult::TestFailed,
            "2" => StepResult::Busted,
            "3" => StepResult::Skipped,
            "4" => StepResult::Exception,
            "5" => StepResult::Retry,
            "6" => StepResult::UserCancel,
            _ => StepResult::Unknown
        }
    }
}

impl ToJson for StepResult {
    fn to_json(&self) -> Json {
        Json::String(match *self {
            StepResult::Unknown => "unknown",
            StepResult::Success => "success",
            StepResult::TestFailed => "testfailed",
            StepResult::Busted => "busted",
            StepResult::Skipped => "skipped",
            StepResult::Exception => "exception",
            StepResult::Retry => "retry",
            StepResult::UserCancel => "usercancel",
        }.into())
    }
}

impl Encodable for StepResult {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(match *self {
            StepResult::Unknown => "unknown",
            StepResult::Success => "success",
            StepResult::TestFailed => "testfailed",
            StepResult::Busted => "busted",
            StepResult::Skipped => "skipped",
            StepResult::Exception => "exception",
            StepResult::Retry => "retry",
            StepResult::UserCancel => "usercancel",
        })
    }
}

#[derive(Debug, RustcEncodable, Clone)]
struct ErrorLine {
    linenumber: u32,
    line: String
}

impl ErrorLine {
    fn new<S>(line_number: u32, line: S) -> ErrorLine
           where S: Into<String> {
        ErrorLine {
            linenumber: line_number,
            line: line.into()
        }
    }
}


#[derive(RustcEncodable)]
struct StepData {
    steps: Vec<Step>,
    all_errors:Vec<ErrorLine>, //TODO: Try making this a reference to avoid a copy
    errors_truncated: bool
}

impl StepData {
    fn new() -> StepData {
        StepData {
            steps: vec![],
            all_errors: vec![],
            errors_truncated: false
        }
    }
}

#[derive(RustcEncodable, Debug)]
struct Step {
    errors: Vec<ErrorLine>,
    name: String,
    started: Option<String>,
    started_linenumber: u32,
    finished_linenumber: u32,
    finished: Option<String>,
    result: StepResult,
    error_count: u32,
    duration: Option<i64>,
    order: u32
}

impl Step {
    fn new<S>(name: S,
              started: Option<S>,
              started_linenumber: u32,
              order: u32) -> Step
        where S: Into<String> {
        Step {
            errors: vec![],
            name: name.into(),
            started: started.map(|x| x.into()),
            started_linenumber: started_linenumber,
            finished_linenumber: 0,
            finished: None,
            result: StepResult::Unknown,
            error_count: 0,
            duration: None,
            order: order
        }
    }

    fn calculate_duration(&self) -> Option<i64> {
        let date_format = "%Y-%m-%d %H:%M:%S%.f";
        if let (Some(started), Some(finished)) = (self.started.as_ref(),
                                                  self.finished.as_ref()) {
            if let (Ok(start), Ok(end)) = (
                UTC.datetime_from_str(started, date_format),
                UTC.datetime_from_str(finished, date_format)) {
                let duration = end - start;
                Some((f64::from(duration.num_milliseconds() as u32) / 1E3).round() as i64)
            } else {
                None
            }
        } else {
            None
        }
    }
}

enum StepState {
    AwaitingFirstStep,
    StepInProgress(Step),
    StepFinished
}

impl StepState {
    fn unwrap(self) -> Step {
        match self {
            StepState::StepInProgress(x) => x,
            _ => panic!("Tried to unwrap a StepState with no Step")
        }
    }
}

pub struct StepParser {
    artifact: StepData,
    state: StepState,
    step_number: u32,
}

impl StepParser {
    pub fn new() -> StepParser {
        StepParser {
            artifact: StepData::new(),
            state: StepState::AwaitingFirstStep,
            step_number: 0
        }
    }

    fn start_step(&mut self,
                  line_number: u32,
                  name: Option<&str>,
                  timestamp: Option<&str>) {
        self.state = StepState::StepInProgress(
            Step::new(name.unwrap_or("Unnamed step"), timestamp, line_number, self.step_number));
        self.step_number += 1;
    }

    fn end_step(&mut self,
                line_number: u32,
                timestamp: Option<&str>,
                result_code: Option<&str>) {
        let mut step = mem::replace(&mut self.state, StepState::StepFinished).unwrap();

        step.error_count = step.errors.len() as u32;
        step.finished_linenumber = line_number;
        step.finished = timestamp.map(|x| x.into());
        if let Some(code) = result_code {
            step.result = StepResult::from_str(code);
        }
        step.duration = step.calculate_duration();
        if step.error_count > PARSER_MAX_STEP_ERROR_LINES as u32 {
            step.errors.truncate(PARSER_MAX_STEP_ERROR_LINES as usize);
            self.artifact.errors_truncated =  true
        }
        self.artifact.all_errors.extend(step.errors.iter().map(|x| x.clone()));
        self.artifact.steps.push(step)
    }

    fn parse_error(&mut self, line: &str, line_number: u32, trimmed: &str) {
        match self.state {
            StepState::AwaitingFirstStep |
            StepState::StepFinished => self.start_step(line_number, None, None),
            StepState::StepInProgress(_) => {}
        }
        //TODO: maybe copy the sub-parser design?
        if self.is_error_line(trimmed) {
            if let StepState::StepInProgress(ref mut state) = self.state {
                state.errors.push(ErrorLine::new(line_number, line));
            }
        }
    }

    fn is_error_line(&self, line: &str) -> bool {
        let matches = RE_ERRORLINE_SET_1.matches(line);
        if matches.matched(0) {
            return false
        } else if matches.matched(1) {
            return true
        }

        let trimmed = if let Some((_, end)) = RE_MOZHARNESS_PREFIX.find(line) {
            &line[end..]
        } else {
            line
        };

        if RE_ERROR_EXCLUDE.is_match(trimmed) {
            return false;
        }

        RE_ERROR_TERMS.is_match(trimmed)
    }
}

impl LogParser for StepParser {
    fn parse_line(&mut self, line: &str, line_number: u32) {
        let trimmed = line.trim_left();

        if trimmed.is_empty() {
            return;
        }

        match self.state {
            StepState::AwaitingFirstStep => {
                if RE_HEADER_LINE.is_match(trimmed) {
                    return
                }
            }
            _ => {}
        }


        if !RE_STEP_MARKER.is_match(trimmed) {
            self.parse_error(line, line_number, trimmed);
        }
        else {
            let step_marker_match = RE_STEP_MARKER.captures(trimmed)
                .expect("Match was found, but got no captures");

            if &step_marker_match["marker_type"] == "Started" {
                if let StepState::StepInProgress(_) = self.state {
                    self.end_step(line_number, None, None);
                }
                self.start_step(line_number,
                                step_marker_match.name("name"),
                                step_marker_match.name("timestamp"));
            } else {
                if let StepState::StepInProgress(_) = self.state {
                    self.end_step(line_number,
                                  step_marker_match.name("timestamp"),
                                  step_marker_match.name("result_code"));
                }
            }
        }
    }

    fn finish_parse(&mut self, last_line_number: u32) {
        match self.state {
            StepState::StepInProgress(_) => self.end_step(last_line_number, None, None),
            _ => {}
        }
    }

    fn clear(&mut self) {
        self.state = StepState::AwaitingFirstStep;
        self.artifact = StepData::new(); 
        self.step_number = 0;
    }

    fn get_artifact(&mut self) -> String {
        let artifact = mem::replace(&mut self.artifact, StepData::new());
        json::encode(&StepDataArtifact::new(artifact)).unwrap()
    }
}

#[derive(RustcEncodable)]
struct StepDataArtifact {
    step_data: StepData
}

impl StepDataArtifact {
    fn new(step_data: StepData) -> StepDataArtifact {
        StepDataArtifact {
            step_data: step_data
        }
    }
}
