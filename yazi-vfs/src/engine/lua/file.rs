use std::{io::{self, ErrorKind, SeekFrom}, ops::{Deref, DerefMut}, pin::Pin, sync::Arc, task::{Context, Poll, ready}};

use mlua::BString;
use tokio::{io::{AsyncRead, AsyncSeek, AsyncWrite, AsyncWriteExt, ReadBuf}, sync::mpsc, task::JoinHandle};
use tokio_util::sync::PollSender;
use yazi_config::vfs::ServiceLua;
use yazi_fs::{engine::Demand, stat::Stat};
use yazi_runner::{RUNNER, provider::{Handle, ProvideChunk, ProvideJob}};
use yazi_shared::{id::Id, url::{AsUrl, Url, UrlBuf, UrlLike}};
use yazi_shim::fs;

type Fut<T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'static>>;

pub struct File {
	url:     UrlBuf,
	handle:  Handle,
	service: Arc<ServiceLua>,
	demand:  Demand,

	read:          Option<ReadState>,
	seek:          Option<SeekState>,
	write:         Option<WriteState>,
	shutdown:      Option<ShutdownState>,
	shutdown_done: bool,
}

impl Deref for File {
	type Target = Handle;

	fn deref(&self) -> &Self::Target { &self.handle }
}

impl DerefMut for File {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.handle }
}

impl Drop for File {
	fn drop(&mut self) {
		let (mut handle, service, url) = (self.handle.clone(), self.service.clone(), self.to_url());
		let shutdown =
			self.shutdown.take().unwrap_or_else(|| ShutdownState::new(self.write.take(), handle.offset));

		tokio::spawn(async move {
			handle.offset = shutdown.await.0;
			let _ = RUNNER.provide(service, ProvideJob::Close { url, handle }).await.ok();
		});
	}
}

impl AsUrl for File {
	fn as_url(&self) -> Url<'_> { self.url.as_url() }
}

impl UrlLike for File {}

impl File {
	pub(super) fn new(
		url: impl Into<UrlBuf>,
		service: Arc<ServiceLua>,
		handle: Handle,
		demand: Demand,
	) -> Self {
		Self {
			url: url.into(),
			handle,
			service,
			demand,

			read: None,
			seek: None,
			write: None,
			shutdown: None,
			shutdown_done: false,
		}
	}

	pub(crate) async fn set_len(&self, size: u64) -> io::Result<()> {
		if self.seekless {
			return Err(ErrorKind::Unsupported.into());
		} else if self.shutdown.is_some() {
			return Err(ErrorKind::BrokenPipe.into());
		}

		let job = ProvideJob::SetLen { url: self.to_url(), size, handle: self.handle.clone() };
		Ok(RUNNER.provide(self.service.clone(), job).await.ok()?)
	}

	pub(crate) async fn set_attrs(&self, attrs: yazi_fs::engine::Attrs) -> io::Result<()> {
		if self.shutdown.is_some() {
			return Err(ErrorKind::BrokenPipe.into());
		}

		let job = ProvideJob::SetAttrs { url: self.to_url(), attrs, handle: Some(self.handle.clone()) };
		Ok(RUNNER.provide(self.service.clone(), job).await.ok()?)
	}

	pub(crate) async fn metadata(&self) -> io::Result<Stat> {
		if self.shutdown.is_some() {
			return Err(ErrorKind::BrokenPipe.into());
		}

		let job = ProvideJob::Metadata { url: self.to_url(), handle: Some(self.handle.clone()) };
		Ok(RUNNER.provide(self.service.clone(), job).await.0?)
	}

	pub(crate) async fn file(&self) -> io::Result<yazi_fs::file::File> {
		if self.shutdown.is_some() {
			return Err(ErrorKind::BrokenPipe.into());
		}

		let job = ProvideJob::File { url: self.to_url(), handle: Some(self.handle.clone()) };
		Ok(RUNNER.provide(self.service.clone(), job).await.0?)
	}

	pub(crate) async fn into_file(mut self) -> io::Result<yazi_fs::file::File> {
		self.shutdown().await?;
		self.file().await
	}

	fn send_impl(
		&mut self,
		cx: &mut Context<'_>,
		len: usize,
		bytes: impl FnOnce() -> Vec<u8>,
	) -> Poll<io::Result<usize>> {
		if self.shutdown_done || self.shutdown.is_some() {
			return Poll::Ready(Err(ErrorKind::BrokenPipe.into()));
		} else if !self.demand.append && !self.demand.write {
			return Poll::Ready(Err(ErrorKind::PermissionDenied.into()));
		}

		ready!(self.flush_impl(cx))?;
		if len == 0 {
			return Poll::Ready(Ok(0));
		}

		let from = self.offset;
		if self.write.is_none() {
			self.write = Some(WriteState::new(self));
		}

		let write = self.write.as_mut().unwrap();
		if ready!(write.data_tx.poll_reserve(cx)).is_err() {
			write.error = Some(ErrorKind::BrokenPipe.into());
			return Poll::Ready(Err(ErrorKind::BrokenPipe.into()));
		}

		let chunk = match ProvideChunk::new(from, bytes()) {
			Ok(chunk) => chunk,
			Err(e) => {
				write.error = Some(fs::Error::from(&e));
				return Poll::Ready(Err(e));
			}
		};

		let to = chunk.to;
		if write.data_tx.send_item(chunk).is_err() {
			write.error = Some(ErrorKind::BrokenPipe.into());
			return Poll::Ready(Err(ErrorKind::BrokenPipe.into()));
		}

		(write.pending, self.read, self.offset) = (true, None, to);
		Poll::Ready(Ok(len))
	}

	fn flush_impl(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		let Some(write) = &mut self.write else { return Poll::Ready(Ok(())) };
		if let Some(e) = &write.error {
			return Poll::Ready(Err(e.into()));
		} else if !write.pending {
			return Poll::Ready(Ok(()));
		}

		match ready!(write.ack_rx.poll_recv(cx)).unwrap_or_else(|| Err(ErrorKind::BrokenPipe.into())) {
			Ok(offset) => {
				write.pending = false;
				self.handle.offset = offset.get();
				Poll::Ready(Ok(()))
			}
			Err(e) => {
				write.error = Some(fs::Error::from(&e));
				Poll::Ready(Err(e))
			}
		}
	}

	fn shutdown_impl(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		if self.shutdown_done {
			return Poll::Ready(Ok(()));
		}

		if self.shutdown.is_none() {
			self.read = None;
			self.shutdown = Some(ShutdownState::new(self.write.take(), self.offset));
		}

		let (offset, result) = ready!(Pin::new(self.shutdown.as_mut().unwrap()).poll(cx));
		self.shutdown = None;
		self.shutdown_done = true;
		self.handle.offset = offset;

		Poll::Ready(result)
	}
}

impl AsyncRead for File {
	fn poll_read(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<io::Result<()>> {
		let me = self.get_mut();
		if me.shutdown.is_some() {
			return Poll::Ready(Err(ErrorKind::BrokenPipe.into()));
		} else if !me.demand.read {
			return Poll::Ready(Err(ErrorKind::PermissionDenied.into()));
		} else if buf.remaining() == 0 {
			return Poll::Ready(Ok(()));
		}

		ready!(me.flush_impl(cx))?;
		if me.read.as_ref().is_some_and(|read| read.done) {
			me.read = None;
		}
		if me.read.is_none() {
			me.read = Some(ReadState::new(me));
		}

		let read = me.read.as_mut().unwrap();
		loop {
			if read.fill(buf, &mut me.handle.offset)? {
				return Poll::Ready(Ok(()));
			} else if read.done {
				return Poll::Ready(Ok(()));
			}

			let Some(result) = ready!(read.rx.poll_recv(cx)) else {
				read.done = true;
				return Poll::Ready(Ok(()));
			};

			match result {
				Err(e) => {
					read.done = true;
					return Poll::Ready(Err(e));
				}
				Ok(bytes) => {
					read.buf = bytes.into();
					read.pos = 0;
				}
			}
		}
	}
}

impl AsyncSeek for File {
	fn start_seek(self: Pin<&mut Self>, position: SeekFrom) -> io::Result<()> {
		if self.shutdown.is_some() {
			return Err(ErrorKind::BrokenPipe.into());
		}

		let me = self.get_mut();
		if me.seekless {
			return Err(ErrorKind::Unsupported.into());
		} else if me.seek.is_some() {
			return Err(io::Error::other("call poll_complete before start_seek"));
		}

		me.read = None;
		me.seek = Some(match position {
			SeekFrom::Start(n) => SeekState::Absolute(n),
			SeekFrom::Current(n) if matches!(&me.write, Some(w) if w.pending) => SeekState::Relative(n),
			SeekFrom::Current(n) => SeekState::current(me.offset, n)?,
			SeekFrom::End(n) => SeekState::end(me, n),
		});

		Ok(())
	}

	fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
		let me = self.get_mut();
		if me.shutdown.is_some() {
			return Poll::Ready(Err(ErrorKind::BrokenPipe.into()));
		}

		ready!(me.flush_impl(cx))?;
		let Some(state) = &mut me.seek else {
			return Poll::Ready(Ok(me.offset));
		};

		let result = match state {
			SeekState::Absolute(n) => Ok(*n),
			SeekState::Relative(n) => me
				.handle
				.offset
				.checked_add_signed(*n)
				.ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "seek overflow")),
			SeekState::End(n, fut) => ready!(fut.as_mut().poll(cx)).and_then(|len| {
				len
					.checked_add_signed(*n)
					.ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "seek overflow"))
			}),
		};
		if let Ok(n) = result {
			me.offset = n;
		}

		me.seek = None;
		Poll::Ready(result)
	}
}

impl AsyncWrite for File {
	fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
		self.get_mut().send_impl(cx, buf.len(), || buf.to_vec())
	}

	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		let me = self.get_mut();
		if me.shutdown.is_some() {
			Poll::Ready(Err(ErrorKind::BrokenPipe.into()))
		} else {
			me.flush_impl(cx)
		}
	}

	fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		self.get_mut().shutdown_impl(cx)
	}

	fn poll_write_vectored(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bufs: &[io::IoSlice<'_>],
	) -> Poll<io::Result<usize>> {
		let len =
			bufs.iter().try_fold(0usize, |n, b| n.checked_add(b.len())).ok_or(ErrorKind::InvalidInput)?;

		self.get_mut().send_impl(cx, len, || {
			let mut bytes = Vec::with_capacity(len);
			for b in bufs {
				bytes.extend_from_slice(b);
			}
			bytes
		})
	}

	fn is_write_vectored(&self) -> bool { true }
}

// --- ReadState
struct ReadState {
	rx:   mpsc::Receiver<io::Result<BString>>,
	buf:  Vec<u8>,
	pos:  usize,
	done: bool,
}

impl ReadState {
	fn new(file: &File) -> Self {
		let service = file.service.clone();
		let job = ProvideJob::Read { url: file.to_url(), handle: file.handle.clone() };

		let (tx, rx) = mpsc::channel(1);
		tokio::spawn(RUNNER.provide_stream(service, job, tx));

		Self { rx, buf: vec![], pos: 0, done: false }
	}

	fn fill(&mut self, buf: &mut ReadBuf<'_>, offset: &mut u64) -> io::Result<bool> {
		if self.pos >= self.buf.len() {
			return Ok(false);
		}

		let len = buf.remaining().min(self.buf.len() - self.pos);
		let next = ProvideChunk::to(*offset, len)?;
		buf.put_slice(&self.buf[self.pos..self.pos + len]);

		*offset = next;
		self.pos += len;
		Ok(true)
	}
}

// --- SeekState
enum SeekState {
	Absolute(u64),
	Relative(i64),
	End(i64, Fut<io::Result<u64>>),
}

impl SeekState {
	fn current(offset: u64, n: i64) -> io::Result<Self> {
		offset
			.checked_add_signed(n)
			.map(Self::Absolute)
			.ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "seek overflow"))
	}

	fn end(file: &File, n: i64) -> Self {
		let service = file.service.clone();
		let job = ProvideJob::Metadata { url: file.to_url(), handle: Some(file.handle.clone()) };

		Self::End(n, Box::pin(async move { Ok(RUNNER.provide::<Stat>(service, job).await.0?.len) }))
	}
}

// --- WriteState
struct WriteState {
	ack_rx:  mpsc::Receiver<io::Result<Id>>,
	data_tx: PollSender<ProvideChunk>,
	worker:  JoinHandle<()>,
	pending: bool,
	error:   Option<fs::Error>,
}

impl WriteState {
	fn new(file: &File) -> Self {
		let (service, url, handle) = (file.service.clone(), file.to_url(), file.handle.clone());

		let (ack_tx, ack_rx) = mpsc::channel(1);
		let (data_tx, data_rx) = mpsc::channel(1);
		let worker = tokio::spawn(RUNNER.provide_stream(
			service,
			ProvideJob::Write { url, handle, stream: data_rx },
			ack_tx,
		));

		Self { ack_rx, data_tx: PollSender::new(data_tx), worker, pending: false, error: None }
	}
}

// --- ShutdownState
struct ShutdownState(Fut<(u64, io::Result<()>)>);

impl ShutdownState {
	fn new(write: Option<WriteState>, mut offset: u64) -> Self {
		Self(Box::pin(async move {
			let result = if let Some(w) = write { Self::finish(w, &mut offset).await } else { Ok(()) };
			(offset, result)
		}))
	}

	async fn finish(write: WriteState, offset: &mut u64) -> io::Result<()> {
		let WriteState { mut ack_rx, data_tx, worker, mut pending, mut error } = write;
		drop(data_tx);

		if error.is_none() {
			while let Some(result) = ack_rx.recv().await {
				match result {
					Ok(_) if !pending => error = Some(ErrorKind::InvalidData.into()),
					Ok(ack) => (pending, *offset) = (false, ack.get()),
					Err(e) => error = Some(e.into()),
				}
				if error.is_some() {
					break;
				}
			}
			if pending {
				error.get_or_insert(ErrorKind::BrokenPipe.into());
			}
		}

		drop(ack_rx);
		let result = worker.await.map_err(io::Error::other);

		error.map_or(result, |e| Err(e.into()))
	}
}

impl Future for ShutdownState {
	type Output = (u64, io::Result<()>);

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		self.get_mut().0.as_mut().poll(cx)
	}
}
