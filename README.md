Easy HTTP Request
====================

[![CI](https://github.com/magiclen/easy-http-request/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/easy-http-request/actions/workflows/ci.yml)

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

## Crates.io

https://crates.io/crates/easy-http-request

## Documentation

https://docs.rs/easy-http-request

## License

[MIT](LICENSE)