use tokio::io::{BufReader, Lines, ReadHalf, WriteHalf};

pub(super) struct Stream;

use tokio::io::AsyncBufReadExt;

#[cfg(unix)]
pub(super) type ClientReader = Lines<BufReader<ReadHalf<tokio::net::UnixStream>>>;
#[cfg(not(unix))]
pub(super) type ClientReader = Lines<BufReader<ReadHalf<tokio::net::TcpStream>>>;

#[cfg(unix)]
pub(super) type ClientWriter = WriteHalf<tokio::net::UnixStream>;
#[cfg(not(unix))]
pub(super) type ClientWriter = WriteHalf<tokio::net::TcpStream>;

#[cfg(unix)]
pub(super) type ServerListener = tokio::net::UnixListener;
#[cfg(not(unix))]
pub(super) type ServerListener = tokio::net::TcpListener;

impl Stream {
	#[cfg(unix)]
	pub(super) async fn connect() -> std::io::Result<(ClientReader, ClientWriter)> {
		let stream = tokio::net::UnixStream::connect(Self::socket_file()).await?;
		let (reader, writer) = tokio::io::split(stream);
		Ok((BufReader::new(reader).lines(), writer))
	}

	#[cfg(not(unix))]
	pub(super) async fn connect() -> std::io::Result<(ClientReader, ClientWriter)> {
		let stream = tokio::net::TcpStream::connect("127.0.0.1:33581").await?;
		let (reader, writer) = tokio::io::split(stream);
		Ok((BufReader::new(reader).lines(), writer))
	}

	#[cfg(unix)]
	pub(super) async fn bind() -> std::io::Result<ServerListener> {
		let p = Self::socket_file();

		tokio::fs::remove_file(&p).await.ok();
		tokio::net::UnixListener::bind(p)
	}

	#[cfg(not(unix))]
	pub(super) async fn bind() -> std::io::Result<ServerListener> {
		tokio::net::TcpListener::bind("127.0.0.1:33581").await
	}

	#[cfg(unix)]
	fn socket_file() -> std::path::PathBuf {
		use std::env::temp_dir;

		use uzers::Users;
		use yazi_shared::USERS_CACHE;

		temp_dir().join(format!(".yazi_dds-{}.sock", USERS_CACHE.get_current_uid()))
	}
}
