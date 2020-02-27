use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter};
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

impl Display for HttpRequestError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            HttpRequestError::UrlParseError(err) => Display::fmt(err, f),
            HttpRequestError::HyperError(err) => Display::fmt(err, f),
            HttpRequestError::IOError(err) => Display::fmt(err, f),
            HttpRequestError::RedirectError(text) => f.write_str(text),
            HttpRequestError::TooLarge => f.write_str("Remote data is too large."),
            HttpRequestError::TimeOut => f.write_str("The connection has timed out."),
            HttpRequestError::LocalNotAllow => f.write_str("Local addresses are not allowed."),
            HttpRequestError::Other(text) => f.write_str(text),
        }
    }
}

impl Error for HttpRequestError {}
