pub trait LogParser {
    fn name(&self) -> &'static str;
    fn parse_line(&mut self, line: &str, line_number: u32);
    fn get_artifact(&mut self) -> String;
    fn finish_parse(&mut self, _last_line_number: u32) {}
    fn complete(&self) -> bool {
        return false
    }
    fn has_artifact(&self) -> bool;
}

