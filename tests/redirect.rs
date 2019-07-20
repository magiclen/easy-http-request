extern crate easy_http_request;

use easy_http_request::DefaultHttpRequest;

#[test]
fn test_redirect() {
    let response = DefaultHttpRequest::get_from_url_str("https://u.magiclen.org/github").unwrap().send().unwrap();

    assert_eq!(200, response.status_code);
}