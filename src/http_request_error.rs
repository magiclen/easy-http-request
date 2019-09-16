use std::io::Error as IOError;

use hyper::error::{Error as HyperError, ParseError};

/// Errors for `HttpRequest`.
#[derive(Debug)]
pub enum HttpRequestError {
    UrlParseError(ParseError),
    HyperError(HyperError),
    IOError(IOError),
    RedirectError(&'static str),
    TooLarge,
    TimeOut,
    LocalNotAllow,
    Other(&'static str),
}

impl From<ParseError> for HttpRequestError {
    #[inline]
    fn from(error: ParseError) -> Self {
        HttpRequestError::UrlParseError(error)
    }
}

impl From<HyperError> for HttpRequestError {
    #[inline]
    fn from(error: HyperError) -> Self {
        HttpRequestError::HyperError(error)
    }
}

impl From<IOError> for HttpRequestError {
    #[inline]
    fn from(error: IOError) -> Self {
        HttpRequestError::IOError(error)
    }
}
