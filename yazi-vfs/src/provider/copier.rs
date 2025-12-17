use std::{io::{self, SeekFrom}, sync::{Arc, atomic::{AtomicU64, Ordering}}};

use futures::{StreamExt, TryStreamExt};
use tokio::{io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter}, select, sync::{mpsc, oneshot}};
use yazi_fs::provider::{Attrs, FileBuilder};
use yazi_shared::url::{Url, UrlBuf};

use crate::provider::{self, Gate};

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
	let acc = Arc::new(AtomicU64::new(0));
	let (from, to) = (Arc::new(from), Arc::new(to));
	let (prog_tx, prog_rx) = mpsc::channel(10);
	let (done_tx, mut done_rx) = oneshot::channel();

	let (acc_, prog_tx_) = (acc.clone(), prog_tx.clone());
	tokio::spawn(async move {
		let init = async {
			let src = provider::open(&*from).await?;
			let cha = src.metadata().await?;

			let dist = provider::create(&*to).await?;
			dist.set_len(cha.len).await?;
			Ok((cha, Some(src), Some(dist)))
		};

		let (cha, mut src, mut dist) = match init.await {
			Ok(r) => r,
			Err(e) => {
				prog_tx_.send(Err(e)).await.ok();
				done_tx.send(()).ok();
				return;
			}
		};

		let chunks = cha.len.div_ceil(PER_CHUNK);
		let it = futures::stream::iter(0..chunks)
			.map(|i| {
				let acc_ = acc_.clone();
				let (from, to) = (from.clone(), to.clone());
				let (src, dist) = (src.take(), dist.take());
				async move {
					let offset = i * PER_CHUNK;
					let take = cha.len.saturating_sub(offset).min(PER_CHUNK);

					let mut src = BufReader::with_capacity(BUF_SIZE, match src {
						Some(f) => f,
						None => provider::open(&*from).await?,
					});
					let mut dist = BufWriter::with_capacity(BUF_SIZE, match dist {
						Some(f) => f,
						None => Gate::default().write(true).open(&*to).await?,
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
						acc_.fetch_add(n as u64, Ordering::SeqCst);
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
			})
			.buffer_unordered(4)
			.try_fold(None, |first, file| async { Ok(first.or(file)) });

		let mut result = select! {
			r = it => r,
			_ = prog_tx_.closed() => return,
		};
		done_tx.send(()).ok();

		let n = acc_.swap(0, Ordering::SeqCst);
		if n > 0 {
			prog_tx_.send(Ok(n)).await.ok();
		}

		if let Ok(Some(file)) = &mut result {
			file.set_attrs(attrs).await.ok();
			file.shutdown().await.ok();
		}

		if let Err(e) = result {
			prog_tx_.send(Err(e)).await.ok();
		} else {
			prog_tx_.send(Ok(0)).await.ok();
		}
	});

	tokio::spawn(async move {
		loop {
			select! {
				_ = &mut done_rx => break,
				_ = prog_tx.closed() => break,
				_ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {},
			}

			let n = acc.swap(0, Ordering::SeqCst);
			if n > 0 {
				prog_tx.send(Ok(n)).await.ok();
			}
		}
	});

	prog_rx
}
