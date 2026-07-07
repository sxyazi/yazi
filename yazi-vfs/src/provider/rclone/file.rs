use std::{io, pin::Pin, process::Stdio, task::{Context, Poll}};

use tokio::{io::{AsyncRead, AsyncSeek, AsyncWrite, ReadBuf}, process::{Child, ChildStdout}};
use yazi_fs::cha::{Cha, ChaMode};

use crate::config::ServiceRclone;

/// A read-only, seekable handle to a remote object.
///
/// rclone has no notion of a persistent file handle, so reads are served by
/// streaming `rclone cat` from the current position; seeking kills the child
/// process and the next read respawns it with `--offset`.
pub struct File {
	config: &'static ServiceRclone,
	target: String,
	/// Object size when known (`rclone lsjson --stat` may report `-1` for
	/// backends that don't expose it, in which case we stream to EOF blindly).
	size:   Option<u64>,
	pos:    u64,
	child:  Option<(Child, ChildStdout)>,
}

impl File {
	pub(super) fn new(config: &'static ServiceRclone, target: String, size: i64) -> Self {
		let size = if size >= 0 { Some(size as u64) } else { None };
		Self { config, target, size, pos: 0, child: None }
	}

	pub(crate) fn cha(&self) -> io::Result<Cha> {
		Ok(Cha {
			mode: ChaMode::try_from(0o100644u16)
				.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
			len: self.size.unwrap_or(0),
			..Default::default()
		})
	}

	fn spawn(&mut self) -> io::Result<()> {
		let mut cmd = super::command(self.config);
		cmd.arg("cat");
		if self.pos > 0 {
			cmd.arg("--offset").arg(self.pos.to_string());
		}
		cmd.arg(&self.target).stderr(Stdio::null());

		let mut child = cmd.spawn()?;
		let stdout =
			child.stdout.take().ok_or_else(|| io::Error::other("No stdout from `rclone cat`"))?;

		self.child = Some((child, stdout));
		Ok(())
	}
}

impl AsyncRead for File {
	fn poll_read(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<io::Result<()>> {
		let me = self.get_mut();

		// Fast path: we've read everything the object claims to hold.
		if me.size == Some(me.pos) {
			return Poll::Ready(Ok(()));
		}
		if me.child.is_none() {
			me.spawn()?;
		}

		let before = buf.filled().len();
		let (_, stdout) = me.child.as_mut().unwrap();
		match Pin::new(stdout).poll_read(cx, buf) {
			Poll::Ready(Ok(())) => {
				let n = (buf.filled().len() - before) as u64;
				me.pos += n;

				// Premature EOF: `rclone cat` closed stdout before delivering the
				// full object. Surface it instead of silently truncating.
				if n == 0
					&& let Some(size) = me.size
					&& me.pos < size
				{
					return Poll::Ready(Err(io::Error::new(
						io::ErrorKind::UnexpectedEof,
						format!("`rclone cat` ended early: got {} of {} bytes", me.pos, size),
					)));
				}
				Poll::Ready(Ok(()))
			}
			polled => polled,
		}
	}
}

impl AsyncSeek for File {
	fn start_seek(self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
		let me = self.get_mut();
		let base = match position {
			io::SeekFrom::Start(n) => Some(n as i128),
			io::SeekFrom::Current(n) => Some(me.pos as i128 + n as i128),
			io::SeekFrom::End(n) => match me.size {
				Some(size) => Some(size as i128 + n as i128),
				None => {
					return Err(io::Error::new(
						io::ErrorKind::Unsupported,
						"Cannot seek from the end of an object with unknown size",
					));
				}
			},
		};

		let pos = base
			.and_then(|p| u64::try_from(p).ok())
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid seek offset"))?;

		if pos != me.pos {
			me.child = None; // Invalidate the stream; the next read respawns at the new offset
			me.pos = pos;
		}
		Ok(())
	}

	fn poll_complete(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
		Poll::Ready(Ok(self.pos))
	}
}

impl AsyncWrite for File {
	fn poll_write(
		self: Pin<&mut Self>,
		_cx: &mut Context<'_>,
		_buf: &[u8],
	) -> Poll<Result<usize, io::Error>> {
		Poll::Ready(Err(super::read_only()))
	}

	fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
		Poll::Ready(Ok(()))
	}

	fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
		Poll::Ready(Ok(()))
	}
}
