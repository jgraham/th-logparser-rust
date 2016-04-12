pub trait LogParser {
    fn parse_line(&mut self, line: &str, line_number: u32);
    fn finish_parse(&mut self, last_line_number: u32);
    fn clear(&mut self);
    fn get_artifact(&mut self) -> String;
}
