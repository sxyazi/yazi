use anyhow::Result;
use tokio::{io, select, sync::{mpsc, oneshot}, time};
use yazi_shared::url::{Component, Url, UrlBuf};

use crate::{cha::Cha, provider::{self, DirReader, FileHolder}};

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

#[inline]
pub fn ok_or_not_found<T: Default>(result: io::Result<T>) -> io::Result<T> {
	match result {
		Ok(t) => Ok(t),
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(T::default()),
		Err(_) => result,
	}
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
					_ = time::sleep(time::Duration::from_secs(3)) => {},
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

pub async fn remove_dir_clean(dir: &UrlBuf) {
	let Ok(mut it) = provider::read_dir(dir).await else { return };

	while let Ok(Some(ent)) = it.next().await {
		if ent.file_type().await.is_ok_and(|t| t.is_dir()) {
			let url = ent.url();
			Box::pin(remove_dir_clean(&url)).await;
			provider::remove_dir(&url).await.ok();
		}
	}

	provider::remove_dir(dir).await.ok();
}

// Find the max common root in a list of urls
// e.g. /a/b/c, /a/b/d       -> /a/b
//      /aa/bb/cc, /aa/dd/ee -> /aa
pub fn max_common_root(urls: &[UrlBuf]) -> usize {
	if urls.is_empty() {
		return 0;
	} else if urls.len() == 1 {
		return urls[0].components().count() - 1;
	}

	let mut it = urls.iter().map(|u| u.parent());
	let Some(first) = it.next().unwrap() else {
		return 0; // The first URL has no parent
	};

	let mut common = first.components().count();
	for parent in it {
		let Some(parent) = parent else {
			return 0; // One of the URLs has no parent
		};

		common = first
			.components()
			.zip(parent.components())
			.take_while(|(a, b)| match (a, b) {
				(Component::Scheme(a), Component::Scheme(b)) => a.covariant(*b),
				(a, b) => a == b,
			})
			.count()
			.min(common);

		if common == 0 {
			break; // No common root found
		}
	}

	common
}

#[cfg(unix)]
#[test]
fn test_max_common_root() {
	fn assert(input: &[&str], expected: &str) {
		use std::{ffi::OsStr, str::FromStr};
		let urls: Vec<_> = input.iter().copied().map(UrlBuf::from_str).collect::<Result<_>>().unwrap();

		let mut comp = urls[0].components();
		for _ in 0..comp.clone().count() - max_common_root(&urls) {
			comp.next_back();
		}
		assert_eq!(comp.os_str(), OsStr::new(expected));
	}

	assert_eq!(max_common_root(&[]), 0);
	assert(&[""], "");
	assert(&["a"], "");

	assert(&["/a"], "/");
	assert(&["/a/b"], "/a");
	assert(&["/a/b/c", "/a/b/d"], "/a/b");
	assert(&["/aa/bb/cc", "/aa/dd/ee"], "/aa");
	assert(&["/aa/bb/cc", "/aa/bb/cc/dd/ee", "/aa/bb/cc/ff"], "/aa/bb");
}
