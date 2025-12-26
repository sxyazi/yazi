use std::mem;

use anyhow::Result;
use yazi_fs::{path::clean_url, provider::{DirReader, FileHolder}};
use yazi_macro::{act, render, succ};
use yazi_parser::cmp::{CmpItem, ShowOpt, TriggerOpt};
use yazi_proxy::CmpProxy;
use yazi_shared::{AnyAsciiChar, BytePredictor, data::Data, natsort, path::{AsPath, PathBufDyn, PathLike}, scheme::{SchemeCow, SchemeLike}, strand::{AsStrand, StrandLike}, url::{UrlBuf, UrlCow, UrlLike}};
use yazi_vfs::provider;

use crate::{Actor, Ctx};

pub struct Trigger;

impl Actor for Trigger {
	type Options = TriggerOpt;

	const NAME: &str = "trigger";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let cmp = &mut cx.cmp;
		if let Some(t) = opt.ticket {
			if t < cmp.ticket {
				succ!();
			}
			cmp.ticket = t;
		}

		let Some((parent, word)) = Self::split_url(&opt.word) else {
			return act!(cmp:close, cx, false);
		};

		if cmp.caches.contains_key(&parent) {
			let ticket = cmp.ticket;
			return act!(cmp:show, cx, ShowOpt { cache: vec![], cache_name: parent, word, ticket });
		}

		let ticket = cmp.ticket;
		tokio::spawn(async move {
			let mut dir = provider::read_dir(&parent).await?;
			let mut cache = vec![];

			// "/" is both a directory separator and the root directory per se
			// As there's no parent directory for the FS root, it is a special case
			if parent.loc() == "/" {
				cache.push(CmpItem { name: Default::default(), is_dir: true });
			}

			while let Ok(Some(ent)) = dir.next().await {
				if let Ok(ft) = ent.file_type().await {
					cache.push(CmpItem { name: ent.name().into_owned(), is_dir: ft.is_dir() });
				}
			}

			if !cache.is_empty() {
				cache
					.sort_unstable_by(|a, b| natsort(a.name.encoded_bytes(), b.name.encoded_bytes(), false));
				CmpProxy::show(ShowOpt { cache, cache_name: parent, word, ticket });
			}

			Ok::<_, anyhow::Error>(())
		});

		succ!(render!(mem::replace(&mut cmp.visible, false)));
	}
}

impl Trigger {
	fn split_url(s: &str) -> Option<(UrlBuf, PathBufDyn)> {
		let sep = if cfg!(windows) {
			AnyAsciiChar::new(b"/\\").unwrap()
		} else {
			AnyAsciiChar::new(b"/").unwrap()
		};

		let (scheme, path) = SchemeCow::parse(s.as_bytes()).ok()?;
		if path.is_empty() && !sep.predicate(s.bytes().last()?) {
			return None; // We don't complete a `sftp://test`, but `sftp://test/`
		}

		// Scheme
		let scheme = scheme.zeroed();
		if scheme.is_local() && path.as_strand() == "~" {
			return None; // We don't complete a `~`, but `~/`
		}

		// Child
		let child = path.rsplit_pred(sep).map_or(path.as_path(), |(_, c)| c).to_owned();

		// Parent
		let url = UrlCow::try_from((scheme.clone().zeroed(), path)).ok()?;
		let abs = if let Some(u) = provider::try_absolute(&url) { u } else { url };
		let parent = abs.loc().try_strip_suffix(&child).ok()?;

		Some((clean_url(UrlCow::try_from((scheme, parent)).ok()?), child))
	}
}

#[cfg(test)]
mod tests {
	use yazi_fs::CWD;
	use yazi_shared::url::UrlLike;

	use super::*;

	fn compare(s: &str, parent: &str, child: &str) {
		let (mut p, c) = Trigger::split_url(s).unwrap();
		if let Ok(u) = p.try_strip_prefix(yazi_fs::CWD.load().as_ref()) {
			p = UrlBuf::Regular(u.as_os().unwrap().into());
		}
		assert_eq!((p, c.to_str().unwrap()), (parent.parse().unwrap(), child));
	}

	#[cfg(unix)]
	#[test]
	fn test_split() {
		yazi_shared::init_tests();
		yazi_fs::init();

		assert_eq!(Trigger::split_url(""), None);
		assert_eq!(Trigger::split_url("sftp://test"), None);
		compare(" ", "", " ");

		compare("/", "/", "");
		compare("//", "/", "");
		compare("///", "/", "");

		compare("/foo", "/", "foo");
		compare("//foo", "/", "foo");
		compare("///foo", "/", "foo");

		compare("/foo/", "/foo/", "");
		compare("//foo/", "/foo/", "");
		compare("/foo/bar", "/foo/", "bar");
		compare("///foo/bar", "/foo/", "bar");

		CWD.set(&"sftp://test".parse::<UrlBuf>().unwrap(), || {});
		compare("sftp://test/a", "sftp://test/.", "a");
		compare("sftp://test//a", "sftp://test//", "a");
		compare("sftp://test2/a", "sftp://test2/.", "a");
		compare("sftp://test2//a", "sftp://test2//", "a");
	}

	#[cfg(windows)]
	#[test]
	fn test_split() {
		yazi_fs::init();
		compare("foo", "", "foo");

		compare(r"foo\", r"foo\", "");
		compare(r"foo\bar", r"foo\", "bar");
		compare(r"foo\bar\", r"foo\bar\", "");

		compare(r"C:\", r"C:\", "");
		compare(r"C:\foo", r"C:\", "foo");
		compare(r"C:\foo\", r"C:\foo\", "");
		compare(r"C:\foo\bar", r"C:\foo\", "bar");
	}
}
