use hyper::client::{Client, Response};
use hyper::header::UserAgent;
use flate2::read::GzDecoder;
use std::io::BufReader;
use std::time::Duration;
use hyper::Result as HyperResult;

pub fn get(url: &str, user_agent: &str, timeout: Option<Duration>) -> HyperResult<Response> {
    let mut client = Client::new();
    client.set_read_timeout(timeout);
    let resp = try!(client.get(url)
        .header(UserAgent(user_agent.into()))
        .send());
    Ok(resp)
}

pub fn read(resp: Response) -> BufReader<GzDecoder<Response>> {
    BufReader::new(GzDecoder::new(resp).unwrap())
}

