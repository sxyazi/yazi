use std::{io, path::PathBuf};

use tokio::{io::{AsyncBufReadExt, BufReader, Lines, ReadHalf, WriteHalf}, sync::OnceCell};
use yazi_fs::{Xdg, create_owned_dir, provider::{Provider, local::Local}};

pub struct Stream;

#[cfg(unix)]
pub type ClientReader = Lines<BufReader<ReadHalf<tokio::net::UnixStream>>>;
#[cfg(windows)]
pub type ClientReader = Lines<BufReader<ReadHalf<tokio::net::TcpStream>>>;

#[cfg(unix)]
pub(super) type ClientWriter = WriteHalf<tokio::net::UnixStream>;
#[cfg(windows)]
pub(super) type ClientWriter = WriteHalf<tokio::net::TcpStream>;

#[cfg(unix)]
pub(super) type ServerListener = tokio::net::UnixListener;
#[cfg(windows)]
pub(super) type ServerListener = WinUnixListener;

impl Stream {
	#[cfg(unix)]
	pub async fn connect() -> io::Result<(ClientReader, ClientWriter)> {
		let stream = tokio::net::UnixStream::connect(Self::socket_file().await?).await?;
		let (reader, writer) = tokio::io::split(stream);
		Ok((BufReader::new(reader).lines(), writer))
	}

	#[cfg(windows)]
	pub async fn connect() -> io::Result<(ClientReader, ClientWriter)> {
		let p = Self::socket_file().await?;
		let uds = tokio::task::spawn_blocking(move || uds_windows::UnixStream::connect(p)).await??;

		let (reader, writer) = tokio::io::split(WinUnixListener::into_tokio(uds)?);
		Ok((BufReader::new(reader).lines(), writer))
	}

	#[cfg(unix)]
	pub(super) async fn bind() -> io::Result<ServerListener> {
		let p = Self::socket_file().await?;

		Local::regular(&p).remove_file().await.ok();
		tokio::net::UnixListener::bind(p)
	}

	#[cfg(windows)]
	pub(super) async fn bind() -> io::Result<ServerListener> {
		let p = Self::socket_file().await?;

		Local::regular(&p).remove_file().await.ok();
		Ok(WinUnixListener(uds_windows::UnixListener::bind(p)?))
	}

	async fn socket_file() -> io::Result<&'static PathBuf> {
		static ONCE: OnceCell<PathBuf> = OnceCell::const_new();
		ONCE
			.get_or_try_init(|| async move {
				let p = Xdg::runtime_dir();
				create_owned_dir(p).await?;

				Ok(p.join(".dds.sock"))
			})
			.await
	}
}

// --- WinUnixListener
#[cfg(windows)]
pub(super) struct WinUnixListener(uds_windows::UnixListener);

#[cfg(windows)]
impl WinUnixListener {
	pub(super) async fn accept(
		&self,
	) -> io::Result<(tokio::net::TcpStream, uds_windows::SocketAddr)> {
		let listener = self.0.try_clone()?;
		let (stream, addr) = tokio::task::spawn_blocking(move || listener.accept()).await??;
		Ok((Self::into_tokio(stream)?, addr))
	}

	fn into_tokio(uds: uds_windows::UnixStream) -> io::Result<tokio::net::TcpStream> {
		use std::os::windows::io::{FromRawSocket, IntoRawSocket};

		let raw = uds.into_raw_socket();
		let std = unsafe { std::net::TcpStream::from_raw_socket(raw) };
		std.set_nonblocking(true)?;

		tokio::net::TcpStream::from_std(std)
	}
}
