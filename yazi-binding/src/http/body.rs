use std::io;

use futures::{Stream, TryStreamExt};
use http_body_util::{BodyExt, Full, StreamBody, combinators::BoxBody};
use hyper::body::{Bytes, Frame};
use reqwest::Body;
use tokio::sync::mpsc;
use tokio_stream::{StreamExt, wrappers::ReceiverStream};

pub(crate) enum HttpBody {
	Bytes(Vec<u8>),
	Stream(mpsc::Receiver<Vec<u8>>),
}

impl From<HttpBody> for Body {
	fn from(body: HttpBody) -> Self {
		match body {
			HttpBody::Bytes(bytes) => bytes.into(),
			HttpBody::Stream(rx) => Body::wrap_stream(HttpBody::stream(rx)),
		}
	}
}

impl HttpBody {
	fn stream(
		rx: mpsc::Receiver<Vec<u8>>,
	) -> impl Stream<Item = Result<Bytes, io::Error>> + Send + 'static {
		ReceiverStream::new(rx).map(|bytes| Ok(bytes.into()))
	}

	pub(super) fn boxed(self) -> BoxBody<Bytes, io::Error> {
		match self {
			Self::Bytes(bytes) => Full::from(bytes).map_err(io::Error::other).boxed(),
			Self::Stream(rx) => StreamBody::new(Self::stream(rx).map_ok(Frame::data)).boxed(),
		}
	}
}
