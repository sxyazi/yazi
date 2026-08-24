use std::io;

use http_body_util::Full;
use hyper::{Request, body::Bytes, client::conn::http1};
use hyper_util::rt::TokioIo;
use reqwest::header::HeaderValue;
use yazi_shim::tokio::net::{UnixStream, UnixStreamExt};

use super::{HttpRequest, HttpResponse};

pub struct HttpTransport<'a> {
	client: &'a reqwest::Client,
}

impl<'a> HttpTransport<'a> {
	pub fn new(client: &'a reqwest::Client) -> Self { Self { client } }

	pub async fn send(self, request: HttpRequest) -> io::Result<HttpResponse> {
		if !request.socket.as_os_str().is_empty() {
			return Self::send_uds(request).await;
		}

		let HttpRequest { url, method, headers, body, .. } = request;
		let mut builder = self.client.request(method, url).headers(headers);
		if let Some(body) = body {
			builder = builder.body(body);
		}
		builder.send().await.map(HttpResponse::new).map_err(io::Error::other)
	}

	async fn send_uds(request: HttpRequest) -> io::Result<HttpResponse> {
		let HttpRequest { url, socket, method, mut headers, body } = request;
		let stream = UnixStream::connect_uds(socket).await?;

		let (mut sender, conn) =
			http1::handshake(TokioIo::new(stream)).await.map_err(io::Error::other)?;
		tokio::spawn(async move {
			conn.await.ok();
		});

		headers.entry(reqwest::header::HOST).or_insert(HeaderValue::from_static("localhost"));
		let mut request = Request::builder()
			.method(method)
			.uri(&url)
			.body(Full::new(Bytes::from(body.unwrap_or_default())))
			.map_err(io::Error::other)?;
		*request.headers_mut() = headers;

		let response = sender.send_request(request).await.map_err(io::Error::other)?;
		Ok(HttpResponse::from_hyper(response, url))
	}
}
