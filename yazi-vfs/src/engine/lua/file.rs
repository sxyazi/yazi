use std::{io::{self, SeekFrom}, pin::Pin, task::{Context, Poll, ready}};

use mlua::BString;
use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite, ReadBuf};
use yazi_fs::{cha::Cha, engine::Demand};
use yazi_runner::{RUNNER, provider::ProviderJob};
use yazi_shared::{event::Cmd, url::UrlBuf};

type Fut<T> = Pin<Box<dyn Future<Output = io::Result<T>> + Send + Sync + 'static>>;

pub struct File {
	url:           UrlBuf,
	pos:           u64,
	run:           &'static Cmd,
	demand:        Demand,
	pending_read:  Option<Fut<BString>>,
	pending_seek:  Option<SeekState>,
	pending_write: Option<Fut<usize>>,
}

enum SeekState {
	NonBlocking(u64),
	Blocking(i64, Fut<u64>),
}

impl File {
	pub(super) fn new(url: impl Into<UrlBuf>, run: &'static Cmd, pos: u64, demand: Demand) -> Self {
		Self {
			url: url.into(),
			pos,
			run,
			demand,
			pending_read: None,
			pending_seek: None,
			pending_write: None,
		}
	}

	pub async fn set_len(&self, size: u64) -> io::Result<()> {
		let url = self.url.clone();
		Ok(RUNNER.provide(self.run, ProviderJob::SetLen { url, size }).await.ok()?)
	}

	pub async fn set_attrs(&self, attrs: yazi_fs::engine::Attrs) -> io::Result<()> {
		let url = self.url.clone();
		Ok(RUNNER.provide(self.run, ProviderJob::SetAttrs { url, attrs }).await.ok()?)
	}

	pub(crate) async fn metadata(&self) -> io::Result<Cha> {
		let url = self.url.clone();
		Ok(RUNNER.provide(self.run, ProviderJob::Metadata { url }).await.0?)
	}

	pub(crate) async fn file(&self) -> io::Result<yazi_fs::file::File> {
		let url = self.url.clone();
		Ok(RUNNER.provide(self.run, ProviderJob::File { url }).await.0?)
	}

	pub(crate) async fn into_file(self) -> io::Result<yazi_fs::file::File> {
		let url = self.url;
		Ok(RUNNER.provide(self.run, ProviderJob::File { url }).await.0?)
	}

	fn write_impl(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bytes: impl FnOnce() -> Vec<u8>,
	) -> Poll<io::Result<usize>> {
		let me = self.get_mut();
		if !me.demand.write {
			return Poll::Ready(Err(io::ErrorKind::PermissionDenied.into()));
		}

		let bytes = bytes();
		if bytes.is_empty() {
			return Poll::Ready(Ok(0));
		}

		if me.pending_write.is_none() {
			let (run, len) = (me.run, bytes.len());
			let job = ProviderJob::Write { url: me.url.clone(), offset: me.pos, bytes };
			me.pending_write = Some(Box::pin(async move {
				RUNNER.provide(run, job).await.ok()?;
				Ok(len)
			}));
		}

		let result = ready!(me.pending_write.as_mut().unwrap().as_mut().poll(cx));
		me.pending_write = None;

		Poll::Ready(result.inspect(|&n| {
			me.pos = me.pos.checked_add(n as u64).expect("offset overflow");
		}))
	}
}

impl AsyncRead for File {
	fn poll_read(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<io::Result<()>> {
		let me = self.get_mut();
		if !me.demand.read {
			return Poll::Ready(Err(io::ErrorKind::PermissionDenied.into()));
		} else if buf.remaining() == 0 {
			return Poll::Ready(Ok(()));
		}

		if me.pending_read.is_none() {
			let run = me.run;
			let job =
				ProviderJob::Read { url: me.url.clone(), offset: me.pos, len: buf.remaining() };
			me.pending_read = Some(Box::pin(async move { Ok(RUNNER.provide(run, job).await.0?) }));
		}

		let result = ready!(me.pending_read.as_mut().unwrap().as_mut().poll(cx));
		me.pending_read = None;

		Poll::Ready(result.map(|bytes| {
			let n = bytes.len().min(buf.remaining());
			buf.put_slice(&bytes[..n]);
			me.pos = me.pos.checked_add(n as u64).expect("offset overflow");
		}))
	}
}

impl AsyncSeek for File {
	fn start_seek(self: Pin<&mut Self>, position: SeekFrom) -> io::Result<()> {
		let me = self.get_mut();
		if me.pending_seek.is_some() {
			return Err(io::Error::other("call poll_complete before start_seek"));
		}

		me.pending_seek = Some(match position {
			SeekFrom::Start(n) => SeekState::NonBlocking(n),
			SeekFrom::Current(n) => me
				.pos
				.checked_add_signed(n)
				.map(SeekState::NonBlocking)
				.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek overflow"))?,
			SeekFrom::End(n) => {
				let run = me.run;
				let job = ProviderJob::Metadata { url: me.url.clone() };
				SeekState::Blocking(
					n,
					Box::pin(async move { Ok(RUNNER.provide::<Cha>(run, job).await.0?.len) }),
				)
			}
		});

		Ok(())
	}

	fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
		let me = self.get_mut();
		let Some(state) = &mut me.pending_seek else {
			return Poll::Ready(Ok(me.pos));
		};

		let result = match state {
			SeekState::NonBlocking(n) => Ok(*n),
			SeekState::Blocking(n, fut) => ready!(fut.as_mut().poll(cx)).and_then(|len| {
				len
					.checked_add_signed(*n)
					.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek overflow"))
			}),
		};
		if let Ok(n) = result {
			me.pos = n;
		}

		me.pending_seek = None;
		Poll::Ready(result)
	}
}

impl AsyncWrite for File {
	fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
		self.write_impl(cx, || buf.to_vec())
	}

	fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		Poll::Ready(Ok(()))
	}

	fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		Poll::Ready(Ok(()))
	}

	fn poll_write_vectored(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bufs: &[io::IoSlice<'_>],
	) -> Poll<io::Result<usize>> {
		let len = bufs.iter().map(|b| b.len()).sum();
		self.write_impl(cx, || {
			let mut bytes = Vec::with_capacity(len);
			for b in bufs {
				bytes.extend_from_slice(b);
			}
			bytes
		})
	}

	fn is_write_vectored(&self) -> bool { true }
}
