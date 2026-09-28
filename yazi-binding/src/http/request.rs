use std::path::PathBuf;

use reqwest::{Method, header::HeaderMap};

use super::HttpSpec;

pub(crate) struct HttpRequest {
	pub(crate) url:     String,
	pub(crate) method:  Method,
	pub(crate) headers: HeaderMap,
	pub(crate) spec:    HttpSpec,
	pub(crate) socket:  PathBuf,
}
