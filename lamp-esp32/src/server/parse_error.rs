#[derive(Debug)]
pub enum ParseError {
    HttpError(httparse::Error),
    Utf8Error(core::str::Utf8Error),
    JsonError(microjson::JSONParsingError),
    ChronoError(chrono::ParseError),
    ValueError,
}

impl From<httparse::Error> for ParseError {
    fn from(err: httparse::Error) -> Self {
        ParseError::HttpError(err)
    }
}
impl From<core::str::Utf8Error> for ParseError {
    fn from(err: core::str::Utf8Error) -> Self {
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

impl From<crate::types::ranges::RangesError> for ParseError {
    fn from(_: crate::types::ranges::RangesError) -> Self {
        ParseError::ValueError
    }
}
