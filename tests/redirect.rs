extern crate easy_http_request;

use easy_http_request::DefaultHttpRequest;

#[test]
fn test_redirect() {
    DefaultHttpRequest::head_from_url_str("https://cloud.magiclen.org").unwrap().send().unwrap();
}