use super::ParseError;

pub struct ResponseBuilder<'a> {
    buffer: &'a mut [u8],
    pos: usize,
}

impl<'a> ResponseBuilder<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, pos: 0 }
    }

    pub fn add(&mut self, text: &str) -> &mut Self {
        let text = text.as_bytes();
        let len = text.len();
        self.buffer[self.pos..self.pos + len].copy_from_slice(text);
        self.pos = self.pos + len;
        self
    }

    pub fn build_response(&mut self) -> &[u8] {
        self.pos = 0;

        let status_line = "HTTP/1.1 200 OK";
        let contents = "{\"response\": \"OK\"}";

        self.add(status_line)
            .add("\r\n")
            .add("Content-Length: ")
            .add(itoa::Buffer::new().format(contents.len()))
            .add("\r\n")
            .add("Content-Type: application/json\r\n")
            .add("\r\n")
            .add(contents)
            .add("\r\n");

        &self.buffer[..self.pos]
    }

    pub fn build_bad_request(&mut self, error: ParseError) -> &[u8] {
        self.pos = 0;

        let status_line = "HTTP/1.1 200 OK";
        let contents_begin = "{\"response\": \"";
        let contents_explanation = match error {
            ParseError::HttpError(_) => "Invalid HTTP request",
            ParseError::Utf8Error(_) => "Invalid UTF-8",
            ParseError::JsonError(_) => "Invalid JSON",
            ParseError::ChronoError(_) => "Invalid DateTime",
            ParseError::ValueError => "Invalid values in request",
        };
        let contents_end = "\"}";

        let contents_len = contents_begin.len() + contents_explanation.len() + contents_end.len();

        self.add(status_line)
            .add("\r\n")
            .add("Content-Length: ")
            .add(itoa::Buffer::new().format(contents_len))
            .add("\r\n")
            .add("Content-Type: application/json\r\n")
            .add("\r\n")
            .add(contents_begin)
            .add(contents_explanation)
            .add(contents_end)
            .add("\r\n");

        &self.buffer[..self.pos]
    }
}
