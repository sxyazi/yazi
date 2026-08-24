use std::{future::Future, io, path::Path};

#[cfg(unix)]
pub type UnixStream = ::tokio::net::UnixStream;
#[cfg(windows)]
pub type UnixStream = ::tokio::net::TcpStream;

pub trait UnixStreamExt: Sized {
	fn connect_uds<P>(path: P) -> impl Future<Output = io::Result<Self>> + Send
	where
		P: AsRef<Path> + Send + 'static;

	#[cfg(windows)]
	fn from_uds(uds: uds_windows::UnixStream) -> io::Result<Self>;
}

impl UnixStreamExt for UnixStream {
	async fn connect_uds<P>(path: P) -> io::Result<Self>
	where
		P: AsRef<Path> + Send + 'static,
	{
		#[cfg(unix)]
		{
			::tokio::net::UnixStream::connect(path).await
		}

		#[cfg(windows)]
		{
			Self::from_uds(
				::tokio::task::spawn_blocking(move || uds_windows::UnixStream::connect(path)).await??,
			)
		}
	}

	#[cfg(windows)]
	fn from_uds(uds: uds_windows::UnixStream) -> io::Result<Self> {
		use std::os::windows::io::{FromRawSocket, IntoRawSocket};

		let raw = uds.into_raw_socket();
		let std = unsafe { std::net::TcpStream::from_raw_socket(raw) };
		std.set_nonblocking(true)?;

		::tokio::net::TcpStream::from_std(std)
	}
}
