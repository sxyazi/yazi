use tokio::io::{BufReader, Lines, ReadHalf, WriteHalf};

pub struct Stream;

use tokio::io::AsyncBufReadExt;

#[cfg(unix)]
pub type ClientReader = Lines<BufReader<ReadHalf<tokio::net::UnixStream>>>;
#[cfg(not(unix))]
pub type ClientReader = Lines<BufReader<ReadHalf<tokio::net::TcpStream>>>;

#[cfg(unix)]
pub type ClientWriter = WriteHalf<tokio::net::UnixStream>;
#[cfg(not(unix))]
pub type ClientWriter = WriteHalf<tokio::net::TcpStream>;

#[cfg(unix)]
pub type ServerListener = tokio::net::UnixListener;
#[cfg(not(unix))]
pub type ServerListener = tokio::net::TcpListener;

impl Stream {
	#[cfg(unix)]
	pub async fn connect() -> std::io::Result<(ClientReader, ClientWriter)> {
		let stream = tokio::net::UnixStream::connect(Self::socket_file()).await?;
		let (reader, writer) = tokio::io::split(stream);
		Ok((BufReader::new(reader).lines(), writer))
	}

	#[cfg(not(unix))]
	pub async fn connect() -> std::io::Result<(ClientReader, ClientWriter)> {
		let stream = tokio::net::TcpStream::connect("127.0.0.1:33581").await?;
		let (reader, writer) = tokio::io::split(stream);
		Ok((BufReader::new(reader).lines(), writer))
	}

	#[cfg(unix)]
	pub async fn bind() -> std::io::Result<ServerListener> {
		let p = Self::socket_file();

		tokio::fs::remove_file(&p).await.ok();
		tokio::net::UnixListener::bind(p)
	}

	#[cfg(not(unix))]
	pub async fn bind() -> std::io::Result<ServerListener> {
		tokio::net::TcpListener::bind("127.0.0.1:33581").await
	}

	#[cfg(unix)]
	pub(super) fn socket_file() -> std::path::PathBuf {
		use uzers::Users;
		use yazi_boot::USERS_CACHE;
		use yazi_shared::Xdg;

		Xdg::cache_dir().join(format!(".dds-{}.sock", USERS_CACHE.get_current_uid()))
	}
}
