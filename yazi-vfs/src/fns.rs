use std::{ffi::OsString, io};

use tokio::{select, sync::{mpsc, oneshot}};
use yazi_fs::cha::Cha;
use yazi_shared::url::{Url, UrlBuf};

use crate::provider;

#[inline]
pub async fn maybe_exists<'a>(url: impl Into<Url<'a>>) -> bool {
	match provider::symlink_metadata(url).await {
		Ok(_) => true,
		Err(e) => e.kind() != io::ErrorKind::NotFound,
	}
}

#[inline]
pub async fn must_be_dir<'a>(url: impl Into<Url<'a>>) -> bool {
	provider::metadata(url).await.is_ok_and(|m| m.is_dir())
}

pub async fn unique_name<F>(u: UrlBuf, append: F) -> io::Result<UrlBuf>
where
	F: Future<Output = bool>,
{
	match provider::symlink_metadata(&u).await {
		Ok(_) => _unique_name(u, append.await).await,
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(u),
		Err(e) => Err(e),
	}
}

async fn _unique_name(mut url: UrlBuf, append: bool) -> io::Result<UrlBuf> {
	let Some(stem) = url.stem().map(|s| s.to_owned()) else {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty file stem"));
	};

	let dot_ext = url.ext().map_or_else(OsString::new, |e| {
		let mut s = OsString::with_capacity(e.len() + 1);
		s.push(".");
		s.push(e);
		s
	});

	let mut i = 1u64;
	let mut name = OsString::with_capacity(stem.len() + dot_ext.len() + 5);
	loop {
		name.clear();
		name.push(&stem);

		if append {
			name.push(&dot_ext);
			name.push(format!("_{i}"));
		} else {
			name.push(format!("_{i}"));
			name.push(&dot_ext);
		}

		url.set_name(&name);
		match provider::symlink_metadata(&url).await {
			Ok(_) => i += 1,
			Err(e) if e.kind() == io::ErrorKind::NotFound => break,
			Err(e) => return Err(e),
		}
	}

	Ok(url)
}

pub fn copy_with_progress(
	from: &UrlBuf,
	to: &UrlBuf,
	cha: Cha,
) -> mpsc::Receiver<Result<u64, io::Error>> {
	let (prog_tx, prog_rx) = mpsc::channel(1);
	let (done_tx, mut done_rx) = oneshot::channel();

	tokio::spawn({
		let (from, to) = (from.clone(), to.clone());
		async move {
			done_tx.send(provider::copy(&from, &to, cha).await).ok();
		}
	});

	tokio::spawn({
		let (prog_tx, to) = (prog_tx.clone(), to.clone());
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

				let len = provider::symlink_metadata(&to).await.map(|m| m.len).unwrap_or(0);
				if len > last {
					prog_tx.send(Ok(len - last)).await.ok();
					last = len;
				}
			}
		}
	});

	prog_rx
}
