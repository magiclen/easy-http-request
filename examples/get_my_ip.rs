extern crate easy_http_request;

use easy_http_request::DefaultHttpRequest;

fn main() {
    let response = DefaultHttpRequest::get_from_url_str("https://tool.magiclen.org/ip").unwrap().send().unwrap();

    println!("{}", String::from_utf8(response.body).unwrap());
}