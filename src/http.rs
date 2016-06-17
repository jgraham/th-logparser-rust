use hyper::client::{Client, Response};
use hyper::header::UserAgent;
use hyper::status::StatusCode;
use flate2::read::GzDecoder;
use std::io::BufReader;
use std::time::Duration;
use logparser::LogParserError;

pub fn get(url: &str, user_agent: &str, timeout: Option<Duration>) -> Result<Response, LogParserError> {
    let mut client = Client::new();
    client.set_read_timeout(timeout);
    let resp = try!(client.get(url)
        .header(UserAgent(user_agent.into()))
        .send());
    match resp.status {
        StatusCode::Ok => Ok(resp),
        _ => Err(LogParserError::Http(resp.status))
    }
}

pub fn read(resp: Response) -> Result<BufReader<GzDecoder<Response>>, LogParserError> {
    Ok(BufReader::new(try!(GzDecoder::new(resp))))
}

