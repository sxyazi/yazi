use std::io;

use tokio::io::{BufReader, Lines, ReadHalf, WriteHalf};

pub struct Stream;

use tokio::io::AsyncBufReadExt;

#[cfg(unix)]
pub type ClientReader = Lines<BufReader<ReadHalf<tokio::net::UnixStream>>>;
#[cfg(not(unix))]
pub type ClientReader = Lines<BufReader<ReadHalf<tokio::net::TcpStream>>>;

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
	pub async fn connect() -> io::Result<(ClientReader, ClientWriter)> {
		let stream = tokio::net::UnixStream::connect(Self::socket_file().await?).await?;
		let (reader, writer) = tokio::io::split(stream);
		Ok((BufReader::new(reader).lines(), writer))
	}

	#[cfg(not(unix))]
	pub async fn connect() -> io::Result<(ClientReader, ClientWriter)> {
		let stream = tokio::net::TcpStream::connect("127.0.0.1:33581").await?;
		let (reader, writer) = tokio::io::split(stream);
		Ok((BufReader::new(reader).lines(), writer))
	}

	#[cfg(unix)]
	pub(super) async fn bind() -> io::Result<ServerListener> {
		use yazi_fs::provider::Provider;

		let p = Self::socket_file().await?;

		yazi_fs::provider::local::Local::regular(&p).remove_file().await.ok();
		tokio::net::UnixListener::bind(p)
	}

	#[cfg(not(unix))]
	pub(super) async fn bind() -> io::Result<ServerListener> {
		tokio::net::TcpListener::bind("127.0.0.1:33581").await
	}

	#[cfg(unix)]
	async fn socket_file() -> io::Result<&'static std::path::PathBuf> {
		use tokio::{fs::DirBuilder, sync::OnceCell};
		use yazi_fs::Xdg;

		static ONCE: tokio::sync::OnceCell<std::path::PathBuf> = OnceCell::const_new();
		ONCE
			.get_or_try_init(|| async move {
				let p = Xdg::runtime_dir();

				#[cfg(unix)]
				DirBuilder::new().mode(0o700).recursive(true).create(p).await?;
				#[cfg(not(unix))]
				DirBuilder::new().recursive(true).create(p).await?;

				Ok(p.join(".dds.sock"))
			})
			.await
	}
}
