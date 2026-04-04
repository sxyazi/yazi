use std::{io::{self, SeekFrom}, sync::{Arc, atomic::{AtomicU64, Ordering}}};

use futures::{StreamExt, TryStreamExt};
use tokio::{io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter}, select, sync::{mpsc, oneshot}};
use yazi_fs::{cha::Cha, provider::{Attrs, FileBuilder}};
use yazi_shared::url::{Url, UrlBuf};

use crate::provider::{self, Gate, RwFile};

const BUF_SIZE: usize = 512 * 1024;
const PER_CHUNK: u64 = 8 * 1024 * 1024;

pub(super) async fn copy_impl(from: Url<'_>, to: Url<'_>, attrs: Attrs) -> io::Result<u64> {
	let src = provider::open(from).await?;
	let dist = provider::create(to).await?;

	let mut reader = BufReader::with_capacity(BUF_SIZE, src);
	let mut writer = BufWriter::with_capacity(BUF_SIZE, dist);
	let written = tokio::io::copy(&mut reader, &mut writer).await?;

	writer.flush().await?;
	writer.get_ref().set_attrs(attrs).await.ok();
	writer.shutdown().await.ok();
	Ok(written)
}

pub(super) fn copy_with_progress_impl(
	from: UrlBuf,
	to: UrlBuf,
	attrs: Attrs,
) -> mpsc::Receiver<io::Result<u64>> {
	let (copier, rx) = ProgressiveCopier::new(from, to, attrs);
	copier.spawn();
	rx
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
		let (done_tx, done_rx) = oneshot::channel();

		tokio::spawn(self.clone().watch(done_rx));
		tokio::spawn(async move {
			if let Err(e) = self.work().await {
				self.prog_tx.send(Err(e)).await.ok();
			}
			done_tx.send(()).ok();
		});
	}

	async fn init(&self) -> io::Result<(Cha, RwFile, RwFile)> {
		let src = provider::open(&self.from).await?;
		let cha = src.metadata().await?;

		let dist = provider::create(&self.to).await?;
		dist.set_len(cha.len).await?;
		Ok((cha, src, dist))
	}

	async fn work(&self) -> io::Result<()> {
		let (cha, src, dist) = self.init().await?;
		let (mut src, mut dist) = (Some(src), Some(dist));

		let chunks = cha.len.div_ceil(PER_CHUNK);
		let it = futures::stream::iter(0..chunks)
			.map(|i| self.map(i, cha, chunks, src.take(), dist.take()))
			.buffer_unordered(4)
			.try_fold(None, |first, file| async { Ok(first.or(file)) });

		let mut result = select! {
			r = it => r,
			_ = self.prog_tx.closed() => return Ok(()),
		};

		let n = self.acc.swap(0, Ordering::SeqCst);
		if n > 0 {
			self.prog_tx.send(Ok(n)).await.ok();
		}

		if let Ok(None) = &mut result {
			result = Ok(dist.take());
		}
		if let Ok(Some(file)) = &mut result {
			file.set_attrs(self.attrs).await.ok();
			file.shutdown().await.ok();
		}

		if let Err(e) = result {
			self.prog_tx.send(Err(e)).await.ok();
		} else {
			self.prog_tx.send(Ok(0)).await.ok();
		}
		Ok(())
	}

	async fn map(
		&self,
		i: u64,
		cha: Cha,
		chunks: u64,
		src: Option<RwFile>,
		dist: Option<RwFile>,
	) -> io::Result<Option<RwFile>> {
		let offset = i * PER_CHUNK;
		let take = cha.len.saturating_sub(offset).min(PER_CHUNK);

		let mut src = BufReader::with_capacity(BUF_SIZE, match src {
			Some(f) => f,
			None => provider::open(&self.from).await?,
		});
		let mut dist = BufWriter::with_capacity(BUF_SIZE, match dist {
			Some(f) => f,
			None => Gate::default().write(true).open(&self.to).await?,
		});

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
		} else if i == chunks - 1 {
			Ok(Some(dist.into_inner()))
		} else {
			dist.shutdown().await.ok();
			Ok(None)
		}
	}

	async fn watch(self: Arc<Self>, mut done_rx: oneshot::Receiver<()>) {
		loop {
			select! {
				_ = &mut done_rx => break,
				_ = self.prog_tx.closed() => break,
				_ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {},
			}

			let n = self.acc.swap(0, Ordering::SeqCst);
			if n > 0 {
				self.prog_tx.send(Ok(n)).await.ok();
			}
		}
	}
}
