use std::io;

use tokio::{select, sync::{mpsc, oneshot}};
use yazi_fs::provider::Attrs;
use yazi_macro::ok_or_not_found;
use yazi_shared::{strand::{StrandBuf, StrandBufLike, StrandLike}, url::{AsUrl, Url, UrlBuf, UrlLike}};

use crate::provider;

#[inline]
pub async fn maybe_exists(url: impl AsUrl) -> bool {
	match provider::symlink_metadata(url).await {
		Ok(_) => true,
		Err(e) => e.kind() != io::ErrorKind::NotFound,
	}
}

#[inline]
pub async fn must_be_dir(url: impl AsUrl) -> bool {
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

	let dot_ext = match url.ext() {
		Some(e) => {
			let mut s = StrandBuf::with_capacity(url.kind(), e.len() + 1);
			s.try_push(".")?;
			s.try_push(e)?;
			s
		}
		None => StrandBuf::default(),
	};

	let mut name = StrandBuf::with_capacity(url.kind(), stem.len() + dot_ext.len() + 5);
	for i in 1u64.. {
		name.clear();
		name.try_push(&stem)?;

		if append {
			name.try_push(&dot_ext)?;
			name.try_push(format!("_{i}"))?;
		} else {
			name.try_push(format!("_{i}"))?;
			name.try_push(&dot_ext)?;
		}

		url.try_set_name(&name)?;
		ok_or_not_found!(provider::symlink_metadata(&url).await, break);
	}

	Ok(url)
}

pub fn copy_with_progress<U, V, A>(
	from: U,
	to: V,
	attrs: A,
) -> mpsc::Receiver<Result<u64, io::Error>>
where
	U: AsUrl,
	V: AsUrl,
	A: Into<Attrs>,
{
	_copy_with_progress(from.as_url(), to.as_url(), attrs.into())
}

fn _copy_with_progress(from: Url, to: Url, attrs: Attrs) -> mpsc::Receiver<Result<u64, io::Error>> {
	let (prog_tx, prog_rx) = mpsc::channel(1);
	let (done_tx, mut done_rx) = oneshot::channel();

	tokio::spawn({
		let (from, to) = (from.to_owned(), to.to_owned());
		async move {
			done_tx.send(provider::copy(from, to, attrs).await).ok();
		}
	});

	tokio::spawn({
		let (prog_tx, to) = (prog_tx.to_owned(), to.to_owned());
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
