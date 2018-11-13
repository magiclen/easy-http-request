use std::fmt::{self, Display, Formatter};

/// The HTTP request method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpRequestMethod {
    /// Get resources.
    GET,
    /// Change resources.
    POST,
    /// Change resources.
    PUT,
    /// Delete resources.
    DELETE,
    /// Only get the headers of resources.
    HEAD,
}

impl HttpRequestMethod {
    pub fn get_str(&self) -> &'static str {
        match self {
            HttpRequestMethod::GET => "GET",
            HttpRequestMethod::POST => "POST",
            HttpRequestMethod::PUT => "PUT",
            HttpRequestMethod::DELETE => "DELETE",
            HttpRequestMethod::HEAD => "HEAD",
        }
    }
}

impl Display for HttpRequestMethod {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(self.get_str())
    }
}