use logparser::{LogParser, LogParserError};
use regex::Regex;
use rustc_serialize::json::{self, Json, ToJson};
use rustc_serialize::{Encodable, Encoder};
use std::mem;

lazy_static! {
    static ref RE_TINDERBOXPRINT: Regex =
        Regex::new(r"TinderboxPrint: ?(?P<line>.*)$").unwrap();

    static ref RE_TALOSRESULT: Regex =
        Regex::new("^TalosResult: ?(?P<value>.*)").unwrap();

    static ref RE_UPLOADED_TO: Regex =
        Regex::new(r#"<a href=['"](?P<url>http(s)?://.*)['"]>(?P<value>.+)</a>: uploaded"#).unwrap();

    static ref RE_LINK_HTML: Regex =
        Regex::new(r#"((?P<title>[A-Za-z/\.0-9-_ ]+): )?<a .*href=['"](?P<url>http(s)?://.+)['"].*>(?P<value>.+)</a>"#).unwrap();

    static ref RE_LINK_TEXT: Regex =
        Regex::new(r"((?P<title>[A-Za-z/\.0-9-_ ]+): )?(?P<url>http(s)?://.*)").unwrap();
    

    static ref TINDERBOX_REGEXPS: [(&'static Regex, Option<&'static str>); 3] =
        [(&*RE_UPLOADED_TO, Some("artifact uploaded")),
         (&*RE_LINK_HTML, None),
         (&*RE_LINK_TEXT, None),];
}

#[derive(Clone, Copy)]
enum ContentType {
    TalosResult,
    Link,
    RawHtml
}

impl Encodable for ContentType {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(match *self {
            ContentType::TalosResult => "TalosResult",
            ContentType::Link => "link",
            ContentType::RawHtml => "raw_html"
        })
    }
}

#[derive(RustcEncodable)]
struct TinderboxData {
    title: Option<String>,
    content_type: ContentType,
    url: Option<String>,
    value: Json
}

impl TinderboxData {
    pub fn new<S>(title: Option<S>, content_type: ContentType, value: Json, url: Option<S>) -> TinderboxData
        where S: Into<String> {
        TinderboxData {
            title: title.map(|x| x.into()),
            content_type: content_type,
            url: url.map(|x| x.into()),
            value: value
        }
    }
}

pub struct TinderboxParser {
    artifact: Vec<TinderboxData>
}

impl TinderboxParser {
    pub fn new() -> TinderboxParser {
        TinderboxParser {
            artifact: vec![]
        }
    }
}

impl LogParser for TinderboxParser {
    fn name(&self) -> &'static str {
        "job_details"
    }
    
    fn parse_line(&mut self, line: &str, _line_number: u32) -> Result<(), LogParserError> {
        if !RE_TINDERBOXPRINT.is_match(line) {
            return Ok(());
        }
        
        let matches = RE_TINDERBOXPRINT.captures(line);
        if matches.is_none() {
            return Ok(());
        }
        let line = matches.unwrap().name("line");
        if line.is_none() {
            return Ok(());
        }
        let line = line.unwrap();
        if RE_TALOSRESULT.is_match(line) {
            let title = "TalosResult";
            //TODO: Need an error here
            let captures = RE_TALOSRESULT.captures(line);
            let json_value = captures
                .expect("RE matched once but not twice")
                .name("value").unwrap_or("{}");
            self.artifact.push(TinderboxData::new(Some(title),
                                                  ContentType::TalosResult,
                                                  try!(Json::from_str(json_value)),
                                                  None));
            return Ok(());
        }

        //TODO: replace this loop with RegexSet
        for &(ref regex, ref default_title) in TINDERBOX_REGEXPS.iter() {
            if regex.is_match(line) {
                let captures = regex.captures(line).unwrap();
                let title = captures.name("title").or(*default_title);
                let mut value = captures.name("value").map(|x| x.to_owned());
                let url = captures.name("url");
                if value.is_none() && url.is_some() {
                    value = url.map(|x| x.to_owned());
                }
                let artifact = TinderboxData::new(title,
                                                  ContentType::Link,
                                                  value.to_json(),
                                                  url);
                self.artifact.push(artifact);
                return Ok(());
            }
        }

        // Default case
        let parts: Vec<&str> = line.splitn(1, "<br/>").collect();
        let (title, value) = if parts.len() == 1 {
            (None, line)
        } else {
            (Some(parts[0]), parts[1])
        };
        let artifact = TinderboxData::new(title, ContentType::RawHtml, value.to_json(),
                                          None);
        self.artifact.push(artifact);
        Ok(())
    }

    fn has_artifact(&self) -> bool {
        self.artifact.len() > 0
    }
    
    fn get_artifact(&mut self) -> String {
        let artifact = mem::replace(&mut self.artifact, vec![]);
        json::encode(&artifact).unwrap()
    }
}
