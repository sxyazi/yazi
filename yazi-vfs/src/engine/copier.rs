use std::{io::{self, SeekFrom}, sync::{Arc, atomic::{AtomicU64, Ordering}}};

use futures::{StreamExt, TryStreamExt};
use tokio::{io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter}, select, sync::mpsc, task::JoinHandle};
use yazi_fs::{engine::{Attrs, FileBuilder, Transmit}, stat::Stat};
use yazi_shared::url::UrlBuf;

use crate::engine::{self, Demand, RwFile};

const BUF_SIZE: usize = 512 * 1024;
const PER_CHUNK: u64 = 8 * 1024 * 1024;

pub(super) fn copy_progressive_impl(from: UrlBuf, to: UrlBuf, attrs: Attrs) -> Transmit {
	let (copier, rx) = ProgressiveCopier::new(from, to, attrs);
	copier.spawn();
	Transmit::new(rx)
}

async fn shutdown_on_error(result: io::Result<()>, file: &mut RwFile) -> io::Result<()> {
	if result.is_err() {
		file.shutdown().await.ok();
	}
	result
}

// --- ProgressiveCopier
struct ProgressiveCopier {
	from:  UrlBuf,
	to:    UrlBuf,
	attrs: Attrs,

	acc:     AtomicU64,
	prog_tx: mpsc::Sender<io::Result<u64>>,
}

impl ProgressiveCopier {
	fn new(from: UrlBuf, to: UrlBuf, attrs: Attrs) -> (Arc<Self>, mpsc::Receiver<io::Result<u64>>) {
		let acc = AtomicU64::new(0);
		let (prog_tx, prog_rx) = mpsc::channel(20);

		(Arc::new(Self { from, to, attrs, acc, prog_tx }), prog_rx)
	}

	fn spawn(self: Arc<Self>) {
		let handle = tokio::spawn(self.clone().work());

		tokio::spawn(self.watch(handle));
	}

	async fn init(&self) -> io::Result<(Stat, RwFile, RwFile)> {
		let src = engine::open(&self.from).await?;
		let stat = src.metadata().await?;

		let dist = RwFile::create(&self.to).await?;
		Ok((stat, src, dist))
	}

	async fn work(self: Arc<Self>) -> io::Result<()> {
		let (stat, src, dist) = self.init().await?;
		let fut = async {
			if src.seekless() || dist.seekless() {
				self.work_sequential(src, dist).await.map(Some)
			} else {
				self.work_random(stat, src, dist).await
			}
		};

		let mut result = select! {
			r = fut => r,
			_ = self.prog_tx.closed() => return Ok(()),
		};

		let n = self.acc.swap(0, Ordering::SeqCst);
		if n > 0 {
			self.prog_tx.send(Ok(n)).await.ok();
		}

		if let Ok(Some(file)) = &mut result {
			file.shutdown().await?;
			file.set_attrs(self.attrs).await.ok();
		}

		result.map(|_| ())
	}

	async fn work_sequential(&self, mut src: RwFile, mut dist: RwFile) -> io::Result<RwFile> {
		let fut = async {
			let mut buf = vec![0u8; 65536];
			loop {
				let n = src.read(&mut buf).await?;
				if n == 0 {
					break;
				}

				dist.write_all(&buf[..n]).await?;
				self.acc.fetch_add(n as u64, Ordering::SeqCst);
			}
			Ok(())
		};

		shutdown_on_error(fut.await, &mut dist).await?;
		Ok(dist)
	}

	async fn work_random(&self, stat: Stat, src: RwFile, dist: RwFile) -> io::Result<Option<RwFile>> {
		dist.set_len(stat.len).await?;

		let (mut src, mut dist) = (Some(src), Some(dist));
		let chunks = stat.len.div_ceil(PER_CHUNK);

		futures::stream::iter(0..chunks)
			.map(|i| self.map(i, stat, chunks, src.take(), dist.take()))
			.buffer_unordered(4)
			.try_fold(None, |first, file| async { Ok(first.or(file)) })
			.await
			.map(|file| file.or(dist))
	}

	async fn map(
		&self,
		i: u64,
		stat: Stat,
		chunks: u64,
		src: Option<RwFile>,
		dist: Option<RwFile>,
	) -> io::Result<Option<RwFile>> {
		let offset = i * PER_CHUNK;
		let take = stat.len.saturating_sub(offset).min(PER_CHUNK);

		let mut src = BufReader::with_capacity(BUF_SIZE, match src {
			Some(f) => f,
			None => engine::open(&self.from).await?,
		});
		let mut dist = BufWriter::with_capacity(BUF_SIZE, match dist {
			Some(f) => f,
			None => Demand::default().write(true).open(&self.to).await?,
		});

		let fut = async {
			src.seek(SeekFrom::Start(offset)).await?;
			dist.seek(SeekFrom::Start(offset)).await?;

			let mut src = src.take(take);
			let mut buf = vec![0u8; 65536];
			let mut copied = 0u64;
			loop {
				let n = src.read(&mut buf).await?;
				if n == 0 {
					break;
				}

				dist.write_all(&buf[..n]).await?;
				copied += n as u64;
				self.acc.fetch_add(n as u64, Ordering::SeqCst);
			}
			dist.flush().await?;

			if copied != take {
				Err(io::Error::other(format!(
					"short copy for chunk {i}: copied {copied} bytes, expected {take}"
				)))
			} else {
				Ok(())
			}
		};

		shutdown_on_error(fut.await, dist.get_mut()).await?;
		if i == chunks - 1 {
			Ok(Some(dist.into_inner()))
		} else {
			dist.shutdown().await?;
			Ok(None)
		}
	}

	async fn watch(self: Arc<Self>, mut done: JoinHandle<io::Result<()>>) {
		loop {
			select! {
				result = &mut done => {
					let result = result.map_err(Into::into).flatten().map(|()| 0);
					self.prog_tx.send(result).await.ok();
					break;
				},
				_ = self.prog_tx.closed() => {
					done.abort();
					break;
				},
				_ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {},
			}

			let n = self.acc.swap(0, Ordering::SeqCst);
			if n > 0 {
				self.prog_tx.send(Ok(n)).await.ok();
			}
		}
	}
}
