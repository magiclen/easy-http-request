use std::collections::HashMap;

/// The http response.
#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}
