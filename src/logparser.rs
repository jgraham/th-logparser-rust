use hyper::Error as HyperError;
use hyper::status::StatusCode;
use rustc_serialize::json::ParserError as JsonParserError;
use std::io::Error as IoError;
use std::error::Error;
use std::fmt;

pub trait LogParser {
    fn name(&self) -> &'static str;
    fn parse_line(&mut self, line: &str, line_number: u32) -> Result<(), LogParserError>;
    fn get_artifact(&mut self) -> String;
    fn finish_parse(&mut self, _last_line_number: u32) {}
    fn complete(&self) -> bool {
        return false
    }
    fn has_artifact(&self) -> bool;
}

#[derive(Debug)]
pub enum LogParserError {
    Network(HyperError),
    Http(StatusCode),
    JsonParse(JsonParserError),
    Io(IoError),
    Other(String)
}

impl LogParserError {
    pub fn name(&self) -> &str {
        match *self {
            LogParserError::Network(_) => "NetworkError",
            LogParserError::Http(_) => "HttpError",
            LogParserError::JsonParse(_) => "JsonError",
            LogParserError::Io(_) => "IoError",
            LogParserError::Other(_) => "OtherError",
        }
    }
}

impl fmt::Display for LogParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for LogParserError {
    fn description(&self) -> &str {
        match *self {
            LogParserError::Network(ref x) => x.description(),
            LogParserError::Http(ref x) => x.canonical_reason().unwrap_or(""),
            LogParserError::JsonParse(ref x) => x.description(),
            LogParserError::Io(ref x) => x.description(),
            LogParserError::Other(ref x) => x,
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            LogParserError::Network(ref x) => x.cause(),
            LogParserError::Http(_) => None,
            LogParserError::JsonParse(ref x) => x.cause(),
            LogParserError::Io(ref x) => x.cause(),
            LogParserError::Other(_) => None
        }
    }
}

impl From<HyperError> for LogParserError {
    fn from(err: HyperError) -> LogParserError {
        LogParserError::Network(err)
    }
}

impl From<JsonParserError> for LogParserError {
    fn from(err: JsonParserError) -> LogParserError {
        LogParserError::JsonParse(err)
    }
}

impl From<IoError> for LogParserError {
    fn from(err: IoError) -> LogParserError {
        LogParserError::Io(err)
    }
}
