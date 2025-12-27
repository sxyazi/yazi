use std::{io, path::PathBuf};

use tokio::{select, sync::{mpsc, oneshot}};

use crate::provider::Attrs;

pub(super) async fn copy_impl(from: PathBuf, to: PathBuf, attrs: Attrs) -> io::Result<u64> {
	#[cfg(any(target_os = "linux", target_os = "android"))]
	{
		use std::os::unix::fs::OpenOptionsExt;

		tokio::task::spawn_blocking(move || {
			let mut opts = std::fs::OpenOptions::new();
			if let Some(mode) = attrs.mode {
				opts.mode(mode.bits() as _);
			}

			let mut reader = std::fs::File::open(from)?;
			let mut writer = opts.write(true).create(true).truncate(true).open(to)?;
			let written = std::io::copy(&mut reader, &mut writer)?;

			if let Some(mode) = attrs.mode {
				writer.set_permissions(mode.into()).ok();
			}
			if let Ok(times) = attrs.try_into() {
				writer.set_times(times).ok();
			}

			Ok(written)
		})
		.await?
	}

	#[cfg(not(any(target_os = "linux", target_os = "android")))]
	{
		tokio::task::spawn_blocking(move || {
			let written = std::fs::copy(from, &to)?;

			if let Ok(times) = attrs.try_into()
				&& let Ok(file) = std::fs::File::options().write(true).open(to)
			{
				file.set_times(times).ok();
			}

			Ok(written)
		})
		.await?
	}
}

pub(super) fn copy_with_progress_impl(
	from: PathBuf,
	to: PathBuf,
	attrs: Attrs,
) -> mpsc::Receiver<Result<u64, io::Error>> {
	let (prog_tx, prog_rx) = mpsc::channel(10);
	let (done_tx, mut done_rx) = oneshot::channel();

	tokio::spawn({
		let to = to.clone();
		async move {
			done_tx.send(copy_impl(from, to, attrs).await).ok();
		}
	});

	tokio::spawn({
		let prog_tx = prog_tx.clone();
		async move {
			let mut last = 0;
			let mut done = None;
			loop {
				select! {
					res = &mut done_rx => done = Some(res.unwrap()),
					_ = prog_tx.closed() => break,
					_ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {},
				}

				match done {
					Some(Ok(len)) => {
						if len > last {
							prog_tx.send(Ok(len - last)).await.ok();
						}
						prog_tx.send(Ok(0)).await.ok();
						break;
					}
					Some(Err(e)) => {
						prog_tx.send(Err(e)).await.ok();
						break;
					}
					None => {}
				}

				let len = tokio::fs::symlink_metadata(&to).await.map(|m| m.len()).unwrap_or(0);
				if len > last {
					prog_tx.send(Ok(len - last)).await.ok();
					last = len;
				}
			}
		}
	});

	prog_rx
}
