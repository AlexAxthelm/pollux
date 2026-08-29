use crux_core::capability::Operation;
use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum HttpOperation {
    FetchFeed { url: String },
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum HttpResult {
    Response { status: u16, body: Vec<u8> },
    Error(String),
}

impl Operation for HttpOperation {
    type Output = HttpResult;
}
