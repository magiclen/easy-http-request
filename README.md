Easy HTTP Request
====================

[![Build Status](https://travis-ci.org/magiclen/easy-http-request.svg?branch=master)](https://travis-ci.org/magiclen/easy-http-request)
[![Build status](https://ci.appveyor.com/api/projects/status/3o434rc48i9g850d/branch/master?svg=true)](https://ci.appveyor.com/project/magiclen/easy-http-request/branch/master)

Easy to send HTTP/HTTPS requests.

## Example

```rust
extern crate easy_http_request;

use easy_http_request::*;

let response = easy_http_request::get("https://magiclen.org", 1 * 1024 * 1024, QUERY_EMPTY, HEADERS_EMPTY).unwrap();

println!("{}", response.status_code);
println!("{:?}", response.headers);
println!("{}", String::from_utf8(response.body).unwrap());
```

## Crates.io

https://crates.io/crates/easy-http-request

## Documentation

https://docs.rs/easy-http-request

## License

[MIT](LICENSE)