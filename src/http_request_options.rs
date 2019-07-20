const DEFAULT_MAX_RESPONSE_BODY_SIZE: usize = 1 * 1024 * 1024;
const DEFAULT_MAX_REDIRECT_COUNT: usize = 5;
const DEFAULT_MAX_CONNECTION_TIME: u64 = 60000;
const DEFAULT_ALLOW_LOCAL: bool = true;

/// Options for `HttpRequest`.
#[derive(Debug, Clone)]
pub struct HttpRequestOptions {
    /// The size limit in bytes of the response body. The default value is `1 * 1024 * 1024` (1 MiB).
    pub max_response_body_size: usize,
    /// The count limit of redirection times. The default value is `5`.
    pub max_redirect_count: usize,
    /// The time limit in milliseconds of a connection. 0 means the time is unlimited. The default value is `60000` (1 minute).
    pub max_connection_time: u64,
    /// Whether to allow to request local URL resources. The default value is `true`.
    pub allow_local: bool,
}

impl Default for HttpRequestOptions {
    #[inline]
    fn default() -> Self {
        HttpRequestOptions {
            max_response_body_size: DEFAULT_MAX_RESPONSE_BODY_SIZE,
            max_redirect_count: DEFAULT_MAX_REDIRECT_COUNT,
            max_connection_time: DEFAULT_MAX_CONNECTION_TIME,
            allow_local: DEFAULT_ALLOW_LOCAL,
        }
    }
}