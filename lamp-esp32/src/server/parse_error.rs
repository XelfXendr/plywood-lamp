use core::str::Utf8Error;

#[derive(Debug)]
pub enum ParseError {
    HttpError(httparse::Error),
    Utf8Error(Utf8Error),
    JsonError(microjson::JSONParsingError),
    ChronoError(chrono::ParseError),
    ValueError,
}

impl From<httparse::Error> for ParseError {
    fn from(err: httparse::Error) -> Self {
        ParseError::HttpError(err)
    }
}
impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> Self {
        ParseError::Utf8Error(err)
    }
}
impl From<microjson::JSONParsingError> for ParseError {
    fn from(err: microjson::JSONParsingError) -> Self {
        ParseError::JsonError(err)
    }
}
impl From<chrono::ParseError> for ParseError {
    fn from(err: chrono::ParseError) -> Self {
        ParseError::ChronoError(err)
    }
}