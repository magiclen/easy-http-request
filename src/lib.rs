/*!
# Easy HTTP Request

Easy to send HTTP/HTTPS requests.

## Example

```rust
extern crate easy_http_request;

use easy_http_request::DefaultHttpRequest;

let response = DefaultHttpRequest::get_from_url_str("https://magiclen.org").unwrap().send().unwrap();

println!("{}", response.status_code);
println!("{:?}", response.headers);
println!("{}", String::from_utf8(response.body).unwrap());
```

More examples are in the `examples` directory.
*/

pub extern crate url;
pub extern crate http;
pub extern crate hyper;
pub extern crate hyper_tls;
pub extern crate tokio_core;
pub extern crate futures;
pub extern crate mime;
pub extern crate slash_formatter;

mod http_request_method;
mod http_request_body;

pub use http_request_method::HttpRequestMethod;
pub use http_request_body::HttpRequestBody;

use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;
use std::io;
use std::string;
use std::fmt::Write;

use tokio_core::reactor;
use hyper::Client;
use hyper::Body;
use hyper::rt::Stream;
use hyper_tls::HttpsConnector;
use http::Request;
use futures::future::Future;
use url::Url;

const DEFAULT_MAX_RESPONSE_BODY_SIZE: usize = 1 * 1024 * 1024;
const DEFAULT_MAX_REDIRECT_COUNT: usize = 5;

/// The http response.
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
    RedirectError(&'static str),
    TooManyRedirect,
    TooLarge,
    Other(&'static str),
}

/// Use strings for query, body and headers.
pub type DefaultHttpRequest = HttpRequest<String, String, String, String, String, String>;

/// Use static string slices for query, body and headers.
pub type StaticHttpRequest = HttpRequest<&'static str, &'static str, &'static str, &'static str, &'static str, &'static str>;

/// The http request sender. See `DefaultHttpRequest` or `StaticHttpRequest`.
pub struct HttpRequest<
    QK = String, QV = String,
    BK = String, BV = String,
    HK = String, HV = String> where QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
                                    BK: Eq + Hash + AsRef<str>, BV: AsRef<str>,
                                    HK: Eq + Hash + AsRef<str>, HV: AsRef<str> {
    pub method: HttpRequestMethod,
    pub url: Url,
    /// The size limit of the response body.
    pub max_response_body_size: usize,
    pub max_redirect_count: usize,
    pub query: Option<HashMap<QK, QV>>,
    pub body: Option<HttpRequestBody<BK, BV>>,
    pub headers: Option<HashMap<HK, HV>>,
}

impl<
    QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
    BK: Eq + Hash + AsRef<str>, BV: AsRef<str>,
    HK: Eq + Hash + AsRef<str>, HV: AsRef<str>> HttpRequest<QK, QV, BK, BV, HK, HV> {
    pub fn new(method: HttpRequestMethod, url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        HttpRequest {
            method,
            url,
            max_response_body_size: DEFAULT_MAX_RESPONSE_BODY_SIZE,
            max_redirect_count: DEFAULT_MAX_REDIRECT_COUNT,
            query: None,
            body: None,
            headers: None,
        }
    }

    pub fn get(url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        Self::new(HttpRequestMethod::GET, url)
    }

    pub fn get_from_url_str<S: AsRef<str>>(url: S) -> Result<HttpRequest<QK, QV, BK, BV, HK, HV>, HttpRequestError> {
        let url = Url::parse(url.as_ref()).map_err(|err| HttpRequestError::UrlParseError(err))?;

        Ok(Self::get(url))
    }

    pub fn post(url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        Self::new(HttpRequestMethod::POST, url)
    }

    pub fn post_from_url_str<S: AsRef<str>>(url: S) -> Result<HttpRequest<QK, QV, BK, BV, HK, HV>, HttpRequestError> {
        let url = Url::parse(url.as_ref()).map_err(|err| HttpRequestError::UrlParseError(err))?;

        Ok(Self::post(url))
    }

    pub fn put(url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        Self::new(HttpRequestMethod::PUT, url)
    }

    pub fn put_from_url_str<S: AsRef<str>>(url: S) -> Result<HttpRequest<QK, QV, BK, BV, HK, HV>, HttpRequestError> {
        let url = Url::parse(url.as_ref()).map_err(|err| HttpRequestError::UrlParseError(err))?;

        Ok(Self::put(url))
    }

    pub fn delete(url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        Self::new(HttpRequestMethod::DELETE, url)
    }

    pub fn delete_from_url_str<S: AsRef<str>>(url: S) -> Result<HttpRequest<QK, QV, BK, BV, HK, HV>, HttpRequestError> {
        let url = Url::parse(url.as_ref()).map_err(|err| HttpRequestError::UrlParseError(err))?;

        Ok(Self::delete(url))
    }

    pub fn head(url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        Self::new(HttpRequestMethod::HEAD, url)
    }

    pub fn head_from_url_str<S: AsRef<str>>(url: S) -> Result<HttpRequest<QK, QV, BK, BV, HK, HV>, HttpRequestError> {
        let url = Url::parse(url.as_ref()).map_err(|err| HttpRequestError::UrlParseError(err))?;

        Ok(Self::head(url))
    }

    /// Send a request and drop this sender.
    pub fn send(self) -> Result<HttpResponse, HttpRequestError> {
        Self::send_request_inner(self.method, self.url, self.max_response_body_size, self.max_redirect_count, &self.query, self.body, &self.headers)
    }

    fn send_request_inner(method: HttpRequestMethod, mut url: Url, max_response_body_size: usize, max_redirect_count: usize, query: &Option<HashMap<QK, QV>>, body: Option<HttpRequestBody<BK, BV>>, headers: &Option<HashMap<HK, HV>>) -> Result<HttpResponse, HttpRequestError> {
        let mut request_builder = Request::builder();

        request_builder.method(method.get_str());
        request_builder.header("User-Agent", concat!("Mozilla/5.0 (Rust; magiclen.org) EasyHyperRequest/", env!("CARGO_PKG_VERSION")));

        if let Some(map) = query {
            let mut query = url.query_pairs_mut();

            for (k, v) in map {
                query.append_pair(k.as_ref(), v.as_ref());
            }
        }

        request_builder.uri(url.to_string());

        match headers {
            Some(map) => {
                for (k, v) in map {
                    request_builder.header(k.as_ref(), v.as_ref());
                }
            }
            None => ()
        }

        let mut has_body = false;

        let request = match body {
            Some(body) => {
                has_body = true;

                match body {
                    HttpRequestBody::Binary { content_type, body } => {
                        request_builder.header("Content-Type", content_type.to_string());
                        request_builder.header("Content-Length", body.len().to_string());
                        request_builder.body(Body::from(body)).map_err(|err| HttpRequestError::HttpError(err))?
                    }
                    HttpRequestBody::Text { content_type, body } => {
                        request_builder.header("Content-Type", content_type.to_string());
                        request_builder.header("Content-Length", body.len().to_string());
                        request_builder.body(Body::from(body.into_bytes())).map_err(|err| HttpRequestError::HttpError(err))?
                    }
                    HttpRequestBody::FormURLEncoded(map) => {
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

                        request_builder.header("Content-Type", "x-www-form-urlencoded");
                        request_builder.header("Content-Length", query.len().to_string());

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

        let mut headers_raw_map = HashMap::new();

        for (name, value) in response.headers() {
            headers_raw_map.insert(name.as_str().to_string(), String::from_utf8(value.as_bytes().to_vec()).map_err(|err| HttpRequestError::FromUtf8Error(err))?);
        }

        let status_code = response.status().as_u16();

        if max_redirect_count > 0 {
            if status_code / 100 == 3 {
                let location_url = match headers_raw_map.get("location") {
                    Some(location) => {
                        match Url::parse(location) {
                            Ok(mut location_url) => {
                                if let Some(host) = url.host().as_ref() {
                                    if location_url.host().is_none() {
                                        let username = url.username();
                                        if !username.is_empty() {
                                            location_url.set_username(username).unwrap();
                                        }

                                        location_url.set_host(Some(&host.to_string())).unwrap();

                                        if let Some(port) = url.port() {
                                            location_url.set_port(Some(port)).unwrap();
                                        }
                                    }
                                }

                                location_url
                            }
                            Err(_) => {
                                let mut location_url = String::new();
                                location_url.push_str(url.scheme());
                                location_url.push_str("://");
                                if let Some(host) = url.host().as_ref() {
                                    let username = url.username();
                                    if !username.is_empty() {
                                        location_url.push_str(username);
                                        location_url.push('@');
                                    }

                                    location_url.push_str(&host.to_string());

                                    if let Some(port) = url.port() {
                                        location_url.write_fmt(format_args!(":{}", port)).unwrap();
                                    }
                                }

                                slash_formatter::concat_with_slash_mut(&mut location_url, location);

                                match Url::parse(&location_url) {
                                    Ok(location_url) => location_url,
                                    Err(_) => return Err(HttpRequestError::RedirectError("Cannot parse the `location` field in headers."))
                                }
                            }
                        }
                    }
                    None => return Err(HttpRequestError::RedirectError("Cannot get the `location` field in headers."))
                };

                match status_code {
                    301 | 302 => {
                        return Self::send_request_inner(HttpRequestMethod::GET, location_url, max_response_body_size, max_redirect_count - 1, query, None, headers);
                    }
                    307 | 308 => {
                        if has_body {
                            eprintln!("Warning: HTTP body's redirection is not supported currently.");
                        }
                        return Self::send_request_inner(method, location_url, max_response_body_size, max_redirect_count - 1, query, None, headers);
                    }
                    _ => {
                        return Err(HttpRequestError::RedirectError("Unsupported redirection status."));
                    }
                }
            }
        }

        let body = core.run(get_body(response.into_body(), max_response_body_size))?;
        // let body = core.run(response.into_body().concat2()).map_err(|err| HttpRequestError::HyperError(err))?.to_vec();

        Ok(HttpResponse {
            status_code,
            headers: headers_raw_map,
            body,
        })
    }
}

impl<
    QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
    BK: Eq + Hash + AsRef<str> + Clone, BV: AsRef<str> + Clone,
    HK: Eq + Hash + AsRef<str>, HV: AsRef<str>> HttpRequest<QK, QV, BK, BV, HK, HV> {
    /// Send a request and preserve this sender so that it can be used again.
    pub fn send_preserved(&self) -> Result<HttpResponse, HttpRequestError> {
        Self::send_request_inner(self.method, self.url.clone(), self.max_response_body_size, self.max_redirect_count, &self.query, self.body.clone(), &self.headers)
    }
}

impl<
    QK: Eq + Hash + AsRef<str> + Clone, QV: AsRef<str> + Clone,
    BK: Eq + Hash + AsRef<str> + Clone, BV: AsRef<str> + Clone,
    HK: Eq + Hash + AsRef<str> + Clone, HV: AsRef<str> + Clone> Clone for HttpRequest<QK, QV, BK, BV, HK, HV> {
    fn clone(&self) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        HttpRequest {
            method: self.method,
            url: self.url.clone(),
            max_response_body_size: self.max_response_body_size,
            max_redirect_count: self.max_redirect_count,
            query: self.query.clone(),
            body: self.body.clone(),
            headers: self.headers.clone(),
        }
    }
}

fn get_body(body: hyper::Body, max_response_body_size: usize) -> Box<Future<Item=Vec<u8>, Error=HttpRequestError>> {
    let mut sum_size = 0;
    let chain = body.then(move |c| {
        let c = c.map_err(|err| HttpRequestError::HyperError(err))?;
        {
            let c_ref = c.as_ref();
            sum_size += c_ref.len();
        }
        let result = if sum_size > max_response_body_size {
            Err(HttpRequestError::TooLarge)
        } else {
            Ok(c)
        };
        result
    });

    let full_body = chain.concat2()
        .map(|chunk| {
            chunk.to_vec()
        });
    Box::new(full_body)
}