use std::collections::HashMap;
use std::hash::Hash;

use mime::Mime;

/// A http request body that you want to send.
#[derive(Debug)]
pub enum HttpRequestBody<BK: Eq + Hash + AsRef<str>, BV: AsRef<str>> {
    Binary {
        content_type: Mime,
        body: Vec<u8>,
    },
    Text {
        content_type: Mime,
        body: String,
    },
    FormURLEncoded(HashMap<BK, BV>),
    // TODO Multi-part
}

impl<BK: Eq + Hash + AsRef<str> + Clone, BV: AsRef<str> + Clone> Clone for HttpRequestBody<BK, BV> {
    fn clone(&self) -> HttpRequestBody<BK, BV> {
        match self {
            HttpRequestBody::Binary {
                content_type,
                body,
            } => {
                HttpRequestBody::Binary {
                    content_type: content_type.clone(),
                    body: body.clone(),
                }
            }
            HttpRequestBody::Text {
                content_type,
                body,
            } => {
                HttpRequestBody::Text {
                    content_type: content_type.clone(),
                    body: body.clone(),
                }
            }
            HttpRequestBody::FormURLEncoded(map) => {
                let mut new_map = HashMap::new();

                for (k, v) in map {
                    new_map.insert(k.clone(), v.clone());
                }

                HttpRequestBody::FormURLEncoded(new_map)
            }
        }
    }
}
