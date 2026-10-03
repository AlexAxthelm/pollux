use crux_core::capability::Operation;
use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum HttpOperation {
    /// Fetches a feed. `etag`/`last_modified` are the validators from the previous
    /// successful fetch; the shell sends them as `If-None-Match`/`If-Modified-Since`
    /// so an unchanged feed answers 304 with no body.
    FetchFeed {
        url: String,
        etag: Option<String>,
        last_modified: Option<String>,
    },
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum HttpResult {
    Response {
        status: u16,
        body: Vec<u8>,
        /// `ETag` response header, verbatim.
        etag: Option<String>,
        /// `Last-Modified` response header, verbatim.
        last_modified: Option<String>,
        /// `Retry-After` normalized by the shell to whole seconds from now (it accepts
        /// both the delay-seconds and HTTP-date forms), so the core needs no date parser.
        retry_after_secs: Option<u64>,
    },
    /// The request failed for a reason on the host's side or in the response (refused
    /// connection, TLS failure, timeout, invalid URL, ...).
    Error(String),
    /// The request never reached the host because the *device* can't reach the network
    /// (offline, cellular data off, roaming off, on a call, or the connection dropped
    /// mid-request). Nothing is wrong with the feed, so refresh doesn't back it off.
    Unreachable(String),
}

impl Operation for HttpOperation {
    type Output = HttpResult;
}
