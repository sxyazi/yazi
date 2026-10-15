use std::{io, panic::AssertUnwindSafe, sync::atomic::{AtomicU64, Ordering}, time::Duration};

use futures::{FutureExt, TryStreamExt};
use tokio::{io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt}, pin, select, sync::mpsc};
use yazi_fs::{engine::{Attrs, Transmit}, stat::Stat};
use yazi_shared::url::UrlBuf;

use crate::engine::{self, FileCursor, RwFile};

const PER_CHUNK: u64 = 8 * 1024 * 1024;

// --- ProgressiveCopier
pub(super) struct ProgressiveCopier {
	from:  UrlBuf,
	to:    UrlBuf,
	attrs: Attrs,

	acc:     AtomicU64,
	prog_tx: mpsc::Sender<io::Result<u64>>,
}

impl ProgressiveCopier {
	pub(super) fn spawn(from: UrlBuf, to: UrlBuf, attrs: Attrs) -> Transmit {
		let (prog_tx, prog_rx) = mpsc::channel(20);
		let copier = Self { from, to, attrs, acc: AtomicU64::new(0), prog_tx };

		tokio::spawn(copier.watch());

		Transmit::new(prog_rx)
	}

	async fn watch(self) {
		// Drop the work before reporting its result; background I/O keeps shared handles alive until it stops.
		let result = {
			let done = AssertUnwindSafe(self.work()).catch_unwind();
			pin!(done);

			loop {
				select! {
					result = &mut done => break result,
					_ = self.prog_tx.closed() => return,
					_ = tokio::time::sleep(Duration::from_secs(3)) => {},
				}

				self.report().await;
			}
		};

		let result = result.map_err(|_| io::Error::other("task panicked")).flatten().map(|()| 0);
		self.prog_tx.send(result).await.ok();
	}

	async fn report(&self) {
		let n = self.acc.swap(0, Ordering::Relaxed);
		if n > 0 {
			self.prog_tx.send(Ok(n)).await.ok();
		}
	}

	async fn work(&self) -> io::Result<()> {
		let src = engine::open(&self.from).await?;
		let stat = if src.seekless() { None } else { Some(src.metadata().await?) };

		let dist = RwFile::create(&self.to).await?;
		let result = match stat {
			Some(stat) if !dist.seekless() => self.work_random(stat, src, dist).await,
			_ => self.work_sequential(src, dist).await,
		};

		self.report().await;
		result?.set_attrs(self.attrs).await.ok();

		Ok(())
	}

	async fn work_sequential(&self, mut src: RwFile, mut dist: RwFile) -> io::Result<RwFile> {
		let result = self.copy_buffered(&mut src, &mut dist).await;
		let shutdown = dist.shutdown().await;

		result.and(shutdown).map(|()| dist)
	}

	async fn work_random(&self, stat: Stat, src: RwFile, dist: RwFile) -> io::Result<RwFile> {
		if stat.len == 0 {
			return self.work_sequential(src, dist).await;
		} else {
			dist.set_len(stat.len).await?;
		}

		let chunks = 0..stat.len.div_ceil(PER_CHUNK);
		futures::stream::iter(chunks.map(Ok))
			.try_for_each_concurrent(4, |i| self.map(i, stat, &src, &dist))
			.await
			.map(|()| dist)
	}

	async fn map(&self, i: u64, stat: Stat, src: &RwFile, dist: &RwFile) -> io::Result<()> {
		let offset = i * PER_CHUNK;
		let take = PER_CHUNK.min(stat.len - offset);

		let mut reader = FileCursor::reader(src, offset).await?.take(take);
		let mut writer = FileCursor::writer(dist, offset).await?;

		let result = self.copy_buffered(&mut reader, &mut writer).await;
		let copied = take - reader.limit();
		drop(reader);

		let shutdown = if result.is_err() {
			// Discard unsent buffered bytes on failure, but still finish the underlying write stream.
			writer.get_mut().shutdown().await
		} else {
			writer.shutdown().await
		};
		drop(writer);

		result.and(shutdown)?;
		if copied != take {
			return Err(io::Error::other(format!(
				"short copy for chunk {i}: copied {copied} bytes, expected {take}"
			)));
		}

		Ok(())
	}

	async fn copy_buffered(
		&self,
		src: &mut (impl AsyncRead + Unpin),
		dist: &mut (impl AsyncWrite + Unpin),
	) -> io::Result<()> {
		let mut buf = vec![0; 128 * 1024];

		loop {
			let n = src.read(&mut buf).await?;
			if n == 0 {
				return Ok(());
			}

			dist.write_all(&buf[..n]).await?;
			self.acc.fetch_add(n as u64, Ordering::Relaxed);
		}
	}
}
