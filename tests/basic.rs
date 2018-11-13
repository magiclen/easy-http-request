extern crate easy_http_request;

use easy_http_request::DefaultHttpRequest;

#[test]
fn test_head() {
    DefaultHttpRequest::head_from_url_str("http://example.com").unwrap().send().unwrap();
    DefaultHttpRequest::head_from_url_str("https://magiclen.org").unwrap().send_preserved().unwrap();
}

#[test]
fn test_get() {
    DefaultHttpRequest::get_from_url_str("http://example.com").unwrap().send().unwrap();
    DefaultHttpRequest::get_from_url_str("https://magiclen.org").unwrap().send_preserved().unwrap();
}