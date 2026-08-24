use std::{io, path::PathBuf};

use tokio::{io::{AsyncBufReadExt, BufReader, Lines, ReadHalf, WriteHalf}, sync::OnceCell};
use yazi_fs::{Xdg, create_owned_dir, engine::{Engine, local::Local}};
use yazi_shim::tokio::net::{UnixStream, UnixStreamExt};

pub struct Stream;

pub type ClientReader = Lines<BufReader<ReadHalf<UnixStream>>>;

pub(super) type ClientWriter = WriteHalf<UnixStream>;

#[cfg(unix)]
pub(super) type ServerListener = tokio::net::UnixListener;
#[cfg(windows)]
pub(super) type ServerListener = WinUnixListener;

impl Stream {
	pub async fn connect() -> io::Result<(ClientReader, ClientWriter)> {
		let stream = UnixStream::connect_uds(Self::socket_file().await?).await?;
		let (reader, writer) = tokio::io::split(stream);
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

		let listener = uds_windows::UnixListener::bind(p)?;
		listener.set_nonblocking(true)?;

		Ok(WinUnixListener(listener))
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
		loop {
			match self.0.accept() {
				Ok((stream, addr)) => return Ok((UnixStream::from_uds(stream)?, addr)),
				Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
					tokio::time::sleep(std::time::Duration::from_millis(20)).await;
				}
				Err(e) => return Err(e),
			}
		}
	}
}
