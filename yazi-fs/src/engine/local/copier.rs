#[cfg(any(test, not(any(target_os = "linux", target_os = "android"))))]
use std::path::Path;
use std::{io, path::PathBuf};

use tokio::{select, sync::{mpsc, oneshot}};

use crate::engine::Attrs;

#[cfg(any(test, not(any(target_os = "linux", target_os = "android"))))]
fn copy_data_only(from: &Path, to: &Path) -> io::Result<u64> {
	let mut reader = std::fs::File::open(from)?;
	let perm = reader.metadata()?.permissions();
	let mut writer = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(to)?;
	let written = std::io::copy(&mut reader, &mut writer)?;
	writer.set_permissions(perm).ok();
	Ok(written)
}

#[cfg(any(test, not(any(target_os = "linux", target_os = "android"))))]
fn copy_or_fallback(from: &Path, to: &Path) -> io::Result<u64> {
	copy_or_fallback_with(from, to, |from, to| std::fs::copy(from, to))
}

#[cfg(any(test, not(any(target_os = "linux", target_os = "android"))))]
fn copy_or_fallback_with<F>(from: &Path, to: &Path, primary: F) -> io::Result<u64>
where
	F: FnOnce(&Path, &Path) -> io::Result<u64>,
{
	primary(from, to).or_else(|_| copy_data_only(from, to))
}

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
			let written = copy_or_fallback(&from, &to)?;

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

pub(super) fn copy_progressive_impl(
	from: PathBuf,
	to: PathBuf,
	attrs: Attrs,
) -> mpsc::Receiver<Result<u64, io::Error>> {
	let (prog_tx, prog_rx) = mpsc::channel(20);
	let (done_tx, mut done_rx) = oneshot::channel();

	tokio::spawn({
		let to = to.clone();
		async move {
			done_tx.send(copy_impl(from, to, attrs).await).ok();
		}
	});

	tokio::spawn(async move {
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
	});

	prog_rx
}

#[cfg(test)]
mod tests {
	use super::*;

	fn scratch() -> PathBuf {
		use std::sync::atomic::{AtomicU64, Ordering};

		static N: AtomicU64 = AtomicU64::new(0);
		let dir = std::env::temp_dir().join(format!(
			"yazi-copier-{}-{}",
			std::process::id(),
			N.fetch_add(1, Ordering::Relaxed)
		));
		std::fs::create_dir_all(&dir).unwrap();
		dir
	}

	fn reject_xattrs(_from: &Path, to: &Path) -> io::Result<u64> {
		std::fs::File::create(to)?;
		Err(io::Error::from_raw_os_error(1))
	}

	#[test]
	fn copy_or_fallback_recovers_when_destination_rejects_xattrs() {
		let dir = scratch();
		let from = dir.join("from");
		let to = dir.join("to");
		std::fs::write(&from, b"copy test\n").unwrap();

		let n = copy_or_fallback_with(&from, &to, reject_xattrs).unwrap();
		assert_eq!(n, 10);
		assert_eq!(std::fs::read(&to).unwrap(), b"copy test\n");

		std::fs::remove_dir_all(dir).unwrap();
	}

	#[test]
	fn copy_or_fallback_copies_regular_file() {
		let dir = scratch();
		let from = dir.join("from");
		let to = dir.join("to");
		std::fs::write(&from, b"hello").unwrap();

		let n = copy_or_fallback(&from, &to).unwrap();
		assert_eq!(n, 5);
		assert_eq!(std::fs::read(&to).unwrap(), b"hello");

		std::fs::remove_dir_all(dir).unwrap();
	}

	#[test]
	fn copy_or_fallback_keeps_successful_primary_copy() {
		let dir = scratch();
		let from = dir.join("from");
		let to = dir.join("to");
		std::fs::write(&from, b"from-source").unwrap();

		let n = copy_or_fallback_with(&from, &to, |_from, to| {
			std::fs::write(to, b"from-primary")?;
			Ok(12)
		})
		.unwrap();
		assert_eq!(n, 12);
		assert_eq!(std::fs::read(&to).unwrap(), b"from-primary");

		std::fs::remove_dir_all(dir).unwrap();
	}
}
