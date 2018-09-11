//! # Easy HTTP Request
//! Easy to send HTTP/HTTPS requests.
//!
//! ## Example
//!
//! ```
//! extern crate easy_http_request;
//!
//! use easy_http_request::*;
//!
//! let response = easy_http_request::get("https://magiclen.org", QUERY_EMPTY, HEADERS_EMPTY).unwrap();
//!
//! println!("{}", response.status_code);
//! println!("{:?}", response.headers);
//! println!("{}", String::from_utf8(response.body).unwrap());
//! ```

pub extern crate url;
pub extern crate http;
pub extern crate hyper;
pub extern crate hyper_tls;
pub extern crate tokio_core;

use tokio_core::reactor;
use hyper::Client;
use hyper::Body;
use hyper::rt::Stream;
use hyper_tls::HttpsConnector;
use http::Request;
use url::Url;
use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;
use std::io;
use std::string;

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub enum HttpRequestError {
    UrlParseError(url::ParseError),
    HttpError(http::Error),
    HyperError(hyper::Error),
    IOError(io::Error),
    FromUtf8Error(string::FromUtf8Error),
    Other(&'static str),
}

pub enum HttpBody<BK: Eq + Hash + AsRef<str>, BV: AsRef<str>> {
    Binary((String, Vec<u8>)),
    Text((String, String)),
    FormURLEncoded(HashMap<BK, BV>),
    // TODO Multi-part
}

pub const QUERY_EMPTY: Option<HashMap<&'static str, &'static str>> = None;
pub const BODY_EMPTY: Option<HttpBody<&'static str, &'static str>> = None;
pub const HEADERS_EMPTY: Option<HashMap<&'static str, &'static str>> = None;

fn request<QK, QV, BK, BV, HK, HV>(method: &str, url: &str, query: Option<HashMap<QK, QV>>, body: Option<HttpBody<BK, BV>>, headers: Option<HashMap<HK, HV>>) -> Result<HttpResponse, HttpRequestError>
    where QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
          BK: Eq + Hash + AsRef<str>, BV: AsRef<str>,
          HK: Eq + Hash + AsRef<str>, HV: AsRef<str> {
    let mut request_builder = Request::builder();

    request_builder.method(method);
    request_builder.header("User-Agent", "my-awesome-agent/1.0");

    match query {
        Some(map) => {
            let mut url = Url::parse(url).map_err(|err| HttpRequestError::UrlParseError(err))?;

            {
                let mut query = url.query_pairs_mut();

                for (k, v) in map {
                    query.append_pair(k.as_ref(), v.as_ref());
                }
            }

            request_builder.uri(url.to_string());
        }
        None => {
            request_builder.uri(url);
        }
    }

    match headers {
        Some(map) => {
            for (k, v) in map {
                request_builder.header(k.as_ref(), v.as_ref());
            }
        }
        None => ()
    }

    let request = match body {
        Some(body) => {
            match body {
                HttpBody::Binary((content_type, vec)) => {
                    request_builder.header("content-type", content_type);
                    request_builder.header("content-length", vec.len().to_string());
                    request_builder.body(Body::from(vec)).map_err(|err| HttpRequestError::HttpError(err))?
                }
                HttpBody::Text((content_type, text)) => {
                    request_builder.header("content-type", content_type);
                    request_builder.header("content-length", text.len().to_string());
                    request_builder.body(Body::from(text.into_bytes())).map_err(|err| HttpRequestError::HttpError(err))?
                }
                HttpBody::FormURLEncoded(map) => {
                    let query = {
                        let mut url = Url::parse("q:").map_err(|err| HttpRequestError::UrlParseError(err))?;
                        {
                            let mut query = url.query_pairs_mut();
                            for (k, v) in map {
                                query.append_pair(k.as_ref(), v.as_ref());
                            }
                        }
                        match url.query() {
                            Some(q) => {
                                q.as_bytes().to_vec()
                            }
                            None => Vec::new()
                        }
                    };

                    request_builder.header("content-type", "x-www-form-urlencoded");
                    request_builder.header("content-length", query.len().to_string());

                    request_builder.body(Body::from(query)).map_err(|err| HttpRequestError::HttpError(err))?
                }
            }
        }
        None => {
            request_builder.body(Body::empty()).map_err(|err| HttpRequestError::HttpError(err))?
        }
    };

    let client = {
        let https = HttpsConnector::new(4).unwrap();
        Client::builder().build::<_, hyper::Body>(https)
    };

    let response = client.request(request);

    let mut core = reactor::Core::new().map_err(|err| HttpRequestError::IOError(err))?;

    let response = core.run(response).map_err(|err| HttpRequestError::HyperError(err))?;

    let mut headers = HashMap::new();

    for (name, value) in response.headers() {
        headers.insert(name.as_str().to_string(), String::from_utf8(value.as_bytes().to_vec()).map_err(|err| HttpRequestError::FromUtf8Error(err))?);
    }

    let status_code = response.status().as_u16();

    let body = core.run(response.into_body().concat2()).map_err(|err| HttpRequestError::HyperError(err))?.to_vec();

    Ok(HttpResponse {
        status_code,
        headers,
        body,
    })
}

pub fn head<QK, QV, HK, HV>(url: &str, query: Option<HashMap<QK, QV>>, headers: Option<HashMap<HK, HV>>) -> Result<HttpResponse, HttpRequestError>
    where QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
          HK: Eq + Hash + AsRef<str>, HV: AsRef<str> {
    request("HEAD", url, query, BODY_EMPTY, headers)
}

pub fn get<QK, QV, HK, HV>(url: &str, query: Option<HashMap<QK, QV>>, headers: Option<HashMap<HK, HV>>) -> Result<HttpResponse, HttpRequestError>
    where QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
          HK: Eq + Hash + AsRef<str>, HV: AsRef<str> {
    request("GET", url, query, BODY_EMPTY, headers)
}

pub fn post<QK, QV, BK, BV, HK, HV>(url: &str, query: Option<HashMap<QK, QV>>, body: Option<HttpBody<BK, BV>>, headers: Option<HashMap<HK, HV>>) -> Result<HttpResponse, HttpRequestError>
    where QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
          BK: Eq + Hash + AsRef<str>, BV: AsRef<str>,
          HK: Eq + Hash + AsRef<str>, HV: AsRef<str> {
    request("POST", url, query, body, headers)
}

pub fn put<QK, QV, BK, BV, HK, HV>(url: &str, query: Option<HashMap<QK, QV>>, body: Option<HttpBody<BK, BV>>, headers: Option<HashMap<HK, HV>>) -> Result<HttpResponse, HttpRequestError>
    where QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
          BK: Eq + Hash + AsRef<str>, BV: AsRef<str>,
          HK: Eq + Hash + AsRef<str>, HV: AsRef<str> {
    request("PUT", url, query, body, headers)
}

pub fn delete<QK, QV, BK, BV, HK, HV>(url: &str, query: Option<HashMap<QK, QV>>, body: Option<HttpBody<BK, BV>>, headers: Option<HashMap<HK, HV>>) -> Result<HttpResponse, HttpRequestError>
    where QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
          BK: Eq + Hash + AsRef<str>, BV: AsRef<str>,
          HK: Eq + Hash + AsRef<str>, HV: AsRef<str> {
    request("DELETE", url, query, body, headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head() {
        head("http://example.com", QUERY_EMPTY, HEADERS_EMPTY).unwrap();
        head("https://magiclen.org", QUERY_EMPTY, HEADERS_EMPTY).unwrap();
    }

    #[test]
    fn test_get() {
        get("http://example.com", QUERY_EMPTY, HEADERS_EMPTY).unwrap();
        get("https://magiclen.org", QUERY_EMPTY, HEADERS_EMPTY).unwrap();
    }
}
