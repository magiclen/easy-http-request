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

#![cfg_attr(feature = "nightly", feature(ip))]
pub extern crate hyper;
pub extern crate hyper_native_tls;
pub extern crate mime;
pub extern crate slash_formatter;
pub extern crate url;

#[macro_use]
extern crate educe;

#[macro_use]
extern crate derivative;

mod http_request_method;
mod http_request_body;
mod http_request_error;
mod http_request_options;
mod http_response;

pub use http_request_method::HttpRequestMethod;
pub use http_request_body::HttpRequestBody;
pub use http_request_error::HttpRequestError;
pub use http_request_options::HttpRequestOptions;
pub use http_response::HttpResponse;

use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;
use std::io::Read;
use std::fmt::Write;
use std::net::{Ipv4Addr, Ipv6Addr};
#[cfg(feature = "nightly")]
use std::net::Ipv6MulticastScope;
use std::str::FromStr;
use std::time::{Instant, Duration};

use url::{Url, Host};

use hyper::client::{Client, Body, RequestBuilder, RedirectPolicy};
use hyper::net::HttpsConnector;
use hyper::method::Method;
use hyper::header::Headers;
use hyper_native_tls::NativeTlsClient;

const BUFFER_SIZE: usize = 512;
const DEFAULT_USER_AGENT: &str = concat!("Mozilla/5.0 (Rust; magiclen.org) EasyHyperRequest/", env!("CARGO_PKG_VERSION"));

/// Use strings for query, body and headers.
pub type DefaultHttpRequest = HttpRequest<String, String, String, String, String, String>;

/// Use static string slices for query, body and headers.
pub type StaticHttpRequest = HttpRequest<&'static str, &'static str, &'static str, &'static str, &'static str, &'static str>;

/// The http request sender. See `DefaultHttpRequest` or `StaticHttpRequest`.
#[derive(Derivative, Educe)]
#[derivative(Clone)]
#[educe(Debug(bound))]
pub struct HttpRequest<
    QK = String, QV = String,
    BK = String, BV = String,
    HK = String, HV = String> where QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
                                    BK: Eq + Hash + AsRef<str>, BV: AsRef<str>,
                                    HK: Eq + Hash + AsRef<str>, HV: AsRef<str> {
    pub method: HttpRequestMethod,
    pub url: Url,
    pub query: Option<HashMap<QK, QV>>,
    pub body: Option<HttpRequestBody<BK, BV>>,
    pub headers: Option<HashMap<HK, HV>>,
    pub options: HttpRequestOptions,
}

impl<
    QK: Eq + Hash + AsRef<str>, QV: AsRef<str>,
    BK: Eq + Hash + AsRef<str>, BV: AsRef<str>,
    HK: Eq + Hash + AsRef<str>, HV: AsRef<str>> HttpRequest<QK, QV, BK, BV, HK, HV> {
    pub fn new(method: HttpRequestMethod, url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        HttpRequest {
            method,
            url,
            query: None,
            body: None,
            headers: None,
            options: HttpRequestOptions::default(),
        }
    }

    pub fn get(url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        Self::new(HttpRequestMethod::GET, url)
    }

    pub fn get_from_url_str<S: AsRef<str>>(url: S) -> Result<HttpRequest<QK, QV, BK, BV, HK, HV>, HttpRequestError> {
        let url = Url::parse(url.as_ref())?;

        Ok(Self::get(url))
    }

    pub fn post(url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        Self::new(HttpRequestMethod::POST, url)
    }

    pub fn post_from_url_str<S: AsRef<str>>(url: S) -> Result<HttpRequest<QK, QV, BK, BV, HK, HV>, HttpRequestError> {
        let url = Url::parse(url.as_ref())?;

        Ok(Self::post(url))
    }

    pub fn put(url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        Self::new(HttpRequestMethod::PUT, url)
    }

    pub fn put_from_url_str<S: AsRef<str>>(url: S) -> Result<HttpRequest<QK, QV, BK, BV, HK, HV>, HttpRequestError> {
        let url = Url::parse(url.as_ref())?;

        Ok(Self::put(url))
    }

    pub fn delete(url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        Self::new(HttpRequestMethod::DELETE, url)
    }

    pub fn delete_from_url_str<S: AsRef<str>>(url: S) -> Result<HttpRequest<QK, QV, BK, BV, HK, HV>, HttpRequestError> {
        let url = Url::parse(url.as_ref())?;

        Ok(Self::delete(url))
    }

    pub fn head(url: Url) -> HttpRequest<QK, QV, BK, BV, HK, HV> {
        Self::new(HttpRequestMethod::HEAD, url)
    }

    pub fn head_from_url_str<S: AsRef<str>>(url: S) -> Result<HttpRequest<QK, QV, BK, BV, HK, HV>, HttpRequestError> {
        let url = Url::parse(url.as_ref())?;

        Ok(Self::head(url))
    }

    /// Send a request and drop this sender.
    pub fn send(self) -> Result<HttpResponse, HttpRequestError> {
        Self::send_request_inner(self.method, self.url, &self.query, &self.body, &self.headers, &self.options, self.options.max_redirect_count)
    }

    /// Send a request and preserve this sender so that it can be used again.
    #[inline]
    pub fn send_preserved(&self) -> Result<HttpResponse, HttpRequestError> {
        Self::send_request_inner(self.method, self.url.clone(), &self.query, &self.body, &self.headers, &self.options, self.options.max_redirect_count)
    }

    fn send_request_inner(method: HttpRequestMethod, mut url: Url, query: &Option<HashMap<QK, QV>>, body: &Option<HttpRequestBody<BK, BV>>, headers: &Option<HashMap<HK, HV>>, options: &HttpRequestOptions, redirection_counter: usize) -> Result<HttpResponse, HttpRequestError> {
        match url.host() {
            Some(host) => {
                if !options.allow_local {
                    match host {
                        Host::Ipv4(ipv4) => {
                            if is_local_ipv4(&ipv4) {
                                return Err(HttpRequestError::LocalNotAllow);
                            }
                        }
                        Host::Ipv6(ipv6) => {
                            if is_local_ipv6(&ipv6) {
                                return Err(HttpRequestError::LocalNotAllow);
                            }
                        }
                        Host::Domain(domain) => {
                            match domain {
                                "localhost" => return Err(HttpRequestError::LocalNotAllow),
                                _ => ()
                            }
                        }
                    }
                }
            }
            None => return Err(HttpRequestError::Other("A valid HTTP URL needs contains a host."))
        }

        if let Some(map) = query {
            let mut query = url.query_pairs_mut();

            for (k, v) in map {
                query.append_pair(k.as_ref(), v.as_ref());
            }
        }

        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);

        let mut client = Client::with_connector(connector);

        if options.max_connection_time > 0 {
            let timeout = Duration::from_millis(options.max_connection_time);

            client.set_read_timeout(Some(timeout));
            client.set_write_timeout(Some(timeout));
        }

        client.set_redirect_policy(RedirectPolicy::FollowNone);

        let mut request: RequestBuilder = client.request(Method::from_str(method.get_str()).unwrap(), url.clone());

        let mut request_headers = Headers::new();

        {
            let has_user_agent = match headers {
                Some(map) => {
                    let mut has_user_agent = false;

                    for (k, v) in map {
                        let name = k.as_ref();
                        let value = v.as_ref().as_bytes();

                        if name.eq_ignore_ascii_case("User-Agent") {
                            has_user_agent = true;
                        }

                        request_headers.append_raw(name.to_string(), value.to_vec());
                    }

                    has_user_agent
                }
                None => false
            };

            if !has_user_agent {
                request_headers.append_raw("User-Agent", DEFAULT_USER_AGENT.as_bytes().to_vec());
            }
        }

        let mut body_owner = None;

        if let Some(body) = body {
            match body {
                HttpRequestBody::Binary { content_type, body } => {
                    request_headers.set_raw("Content-Type", vec![content_type.to_string().into_bytes()]);

                    let body_size = body.len();

                    request_headers.set_raw("Content-Length", vec![body_size.to_string().into_bytes()]);

                    request = request.body(Body::BufBody(body, body_size));
                }
                HttpRequestBody::Text { content_type, body } => {
                    request_headers.set_raw("Content-Type", vec![content_type.to_string().into_bytes()]);

                    let body_size = body.len();

                    request_headers.set_raw("Content-Length", vec![body_size.to_string().into_bytes()]);

                    request = request.body(Body::BufBody(body.as_ref(), body_size));
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

                    request_headers.set_raw("Content-Type", vec![b"x-www-form-urlencoded".to_vec()]);

                    let body_size = query.len();

                    request_headers.set_raw("Content-Length", vec![body_size.to_string().into_bytes()]);

                    body_owner = Some(query);

                    if let Some(body) = body_owner.as_ref() {
                        request = request.body(Body::BufBody(body.as_ref(), body_size));
                    }
                }
            }
        }

        request = request.headers(request_headers);

        let start_time = Instant::now();

        let mut response = request.send()?;

        let u64_max = u64::max_value() as u128;

        if options.max_connection_time > 0 {
            let elapsed = start_time.elapsed();

            let millis = elapsed.as_millis();
            if millis > u64_max || millis as u64 > options.max_connection_time {
                return Err(HttpRequestError::TimeOut);
            }
        }

        let status_code = response.status.to_u16();

        let mut headers_raw_map = HashMap::new();

        for header in response.headers.iter() {
            headers_raw_map.insert(header.name().to_lowercase(), header.value_string());
        }

        if redirection_counter > 0 {
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
                    303 => {
                        drop(headers_raw_map);
                        drop(body_owner);
                        drop(response);
                        drop(client);

                        return Self::send_request_inner(HttpRequestMethod::GET, location_url, query, &None, headers, options, redirection_counter);
                    }
                    301 | 302 | 307 | 308 => {
                        drop(headers_raw_map);
                        drop(body_owner);
                        drop(response);
                        drop(client);

                        return Self::send_request_inner(method, location_url, query, body, headers, options, redirection_counter);
                    }
                    _ => {
                        return Err(HttpRequestError::RedirectError("Unsupported redirection status."));
                    }
                }
            }
        }

        let mut sum_size = 0;
        let mut body = Vec::new();
        let mut buffer = [0u8; BUFFER_SIZE];

        loop {
            let c = response.read(&mut buffer)?;

            if c == 0 {
                break;
            }

            sum_size += c;

            if sum_size > options.max_response_body_size {
                return Err(HttpRequestError::TooLarge);
            }

            body.extend_from_slice(&buffer[0..c]);

            if options.max_connection_time > 0 {
                let elapsed = start_time.elapsed();

                let millis = elapsed.as_millis();
                if millis > u64_max || millis as u64 > options.max_connection_time {
                    return Err(HttpRequestError::TimeOut);
                }
            }
        }

        Ok(HttpResponse {
            status_code,
            headers: headers_raw_map,
            body,
        })
    }
}

#[inline]
fn is_local_ipv4(addr: &Ipv4Addr) -> bool {
    addr.is_private() || addr.is_loopback() || addr.is_link_local() || addr.is_broadcast() || addr.is_documentation() || addr.is_unspecified()
}

#[cfg(not(feature = "nightly"))]
#[inline]
fn is_local_ipv6(addr: &Ipv6Addr) -> bool {
    addr.is_multicast() || addr.is_loopback() || addr.is_unspecified()
}

#[cfg(feature = "nightly")]
#[inline]
fn is_local_ipv6(addr: &Ipv6Addr) -> bool {
    match addr.multicast_scope() {
        Some(Ipv6MulticastScope::Global) => false,
        None => addr.is_multicast() || addr.is_loopback() || addr.is_unicast_link_local() || addr.is_unicast_site_local() || addr.is_unique_local() || addr.is_unspecified() || addr.is_documentation(),
        _ => true
    }
}