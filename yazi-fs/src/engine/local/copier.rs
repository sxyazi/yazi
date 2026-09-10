use std::{fs, io, path::{Path, PathBuf}, time::Duration};

use tokio::{select, sync::mpsc, task, time};

use crate::engine::{Attrs, Transmit};

pub(super) fn copy_progressive(from: PathBuf, to: PathBuf, attrs: Attrs) -> Transmit {
	let (prog_tx, prog_rx) = mpsc::channel(20);

	tokio::spawn(async move {
		let mut initial = tokio::fs::symlink_metadata(&to).await.ok().map(|m| m.len());
		if prog_tx.is_closed() {
			return;
		}

		let mut last = 0;
		let mut done = imp(from, to.clone(), attrs);
		loop {
			select! {
				output = &mut done => {
					match output {
						Ok(Ok(len)) => {
							if len > last {
								prog_tx.send(Ok(len - last)).await.ok();
							}
							prog_tx.send(Ok(0)).await.ok();
						}
						Ok(Err(e)) => _ = prog_tx.send(Err(e)).await,
						Err(e) => _ = prog_tx.send(Err(e.into())).await
					}
					break;
				}
				_ = prog_tx.closed() => {
					done.abort();
					break;
				}
				_ = time::sleep(Duration::from_secs(3)) => {
					let Ok(len) = tokio::fs::symlink_metadata(&to).await.map(|m| m.len()) else { continue };
					if initial == Some(len) {
						continue;
					}

					initial = None;
					if len > last {
						prog_tx.send(Ok(len - last)).await.ok();
						last = len;
					}
				}
			}
		}
	});

	Transmit::new(prog_rx)
}

fn imp(from: PathBuf, to: PathBuf, attrs: Attrs) -> task::JoinHandle<io::Result<u64>> {
	task::spawn_blocking(move || primary_imp(&from, &to, attrs))
}

fn primary_imp(from: &Path, to: &Path, attrs: Attrs) -> io::Result<u64> {
	let written = match fs::copy(from, to) {
		Ok(n) => n,
		#[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
		Err(e) if matches!(e.kind(), io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported) => {
			return fallback_imp(from, to, attrs);
		}
		Err(e) => return Err(e),
	};

	if let Ok(times) = attrs.try_into() {
		yazi_shim::fs::set_times(to, times).ok();
	}
	Ok(written)
}

#[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
fn fallback_imp(from: &Path, to: &Path, attrs: Attrs) -> io::Result<u64> {
	use std::os::unix::fs::OpenOptionsExt;

	let mut opts = fs::OpenOptions::new();
	if let Some(mode) = attrs.mode {
		opts.mode(mode.bits() as _);
	}

	let mut reader = fs::File::open(from)?;
	let mut writer = opts.write(true).create(true).truncate(true).open(to)?;
	let written = io::copy(&mut reader, &mut writer)?;

	if let Some(mode) = attrs.mode {
		writer.set_permissions(mode.into()).ok();
	} else if let Ok(perm) = reader.metadata().map(|m| m.permissions()) {
		writer.set_permissions(perm).ok();
	}
	if let Ok(times) = attrs.try_into() {
		writer.set_times(times).ok();
	}
	Ok(written)
}
