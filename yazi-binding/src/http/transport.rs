use std::{io, sync::LazyLock};

use hyper::{Request, client::conn::http1};
use hyper_util::rt::TokioIo;
use reqwest::{Client, header::{HOST, HeaderValue}};
use yazi_shim::tokio::net::{UnixStream, UnixStreamExt};

use super::{HttpBody, HttpRequest, HttpResponse, HttpSpec};

pub(super) async fn send(request: HttpRequest) -> io::Result<HttpResponse> {
	if !request.socket.as_os_str().is_empty() {
		return send_uds(request).await;
	}

	let HttpRequest { url, method, headers, spec, .. } = request;
	let mut builder = client()?.request(method, url).headers(headers);
	builder = match spec {
		HttpSpec::Multipart(form) => builder.multipart(form.into()),
		HttpSpec::Bytes(bytes) => builder.body(bytes),
		HttpSpec::Raw { rx, .. } => builder.body(HttpBody::Stream(rx)),
		HttpSpec::UrlEncoded(form) => builder.body(form.to_urlencoded()),
	};

	builder.send().await.map(HttpResponse::new).map_err(io::Error::other)
}

async fn send_uds(request: HttpRequest) -> io::Result<HttpResponse> {
	let HttpRequest { url, socket, method, mut headers, spec } = request;
	let stream = UnixStream::connect_uds(socket).await?;

	let (mut sender, conn) =
		http1::handshake(TokioIo::new(stream)).await.map_err(io::Error::other)?;
	tokio::spawn(conn);

	headers.entry(HOST).or_insert(HeaderValue::from_static("localhost"));
	let body = match spec {
		HttpSpec::Multipart(form) => form.into(),
		HttpSpec::Bytes(bytes) => HttpBody::Bytes(bytes).boxed(),
		HttpSpec::Raw { rx, .. } => HttpBody::Stream(rx).boxed(),
		HttpSpec::UrlEncoded(form) => HttpBody::Bytes(form.to_urlencoded().into()).boxed(),
	};

	let mut request =
		Request::builder().method(method).uri(&url).body(body).map_err(io::Error::other)?;
	*request.headers_mut() = headers;

	let response = sender.send_request(request).await.map_err(io::Error::other)?;
	Ok(HttpResponse::from_hyper(response, url))
}

fn client() -> io::Result<&'static Client> {
	static HTTP: LazyLock<Result<Client, reqwest::Error>> = LazyLock::new(|| {
		Client::builder()
			.user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36")
			.build()
	});

	HTTP.as_ref().map_err(|e| io::Error::other(e.to_string()))
}
