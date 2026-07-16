use std::{io, mem};

use anyhow::Result;
use yazi_core::cmp::{CmpItem, CmpOpt};
use yazi_fs::{engine::{DirReader, FileHolder}, path::clean_url};
use yazi_macro::{act, render, succ};
use yazi_parser::cmp::TriggerForm;
use yazi_proxy::CmpProxy;
use yazi_shared::{AnyAsciiChar, BytePredictor, data::Data, natsort, path::{DynPath, PathBufDyn, PathLike}, spec::Spec, strand::{AsStrand, StrandLike}, url::{UrlBuf, UrlCow, UrlLike}};
use yazi_vfs::engine;

use crate::{Actor, Ctx};

pub struct Trigger;

impl Actor for Trigger {
	type Form = TriggerForm;

	const NAME: &str = "trigger";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		if form.ticket.is_some_and(|t| t != cx.cmp.ticket) {
			succ!();
		} else if form.ticket.is_none() {
			cx.cmp.ticket = cx.input.lock().map(|g| g.ticket.current()).unwrap_or_default();
		}

		cx.cmp.handle.take().map(|h| h.abort());
		let Some((parent, word)) = Self::split_url(&form.word) else {
			return act!(cmp:close, cx, false);
		};

		let ticket = cx.cmp.ticket;
		if cx.cmp.caches.contains_key(&parent) {
			return act!(cmp:show, cx, CmpOpt { cache: vec![], cache_name: parent, word, ticket });
		}

		cx.cmp.handle = Some(tokio::spawn(async move {
			let mut dir = engine::read_dir(&parent).await?;
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
					.sort_unstable_by(|a, b| natsort(a.name.encoded_bytes(), b.name.encoded_bytes(), true));
				CmpProxy::show(CmpOpt { cache, cache_name: parent, word, ticket });
			}

			Ok::<_, io::Error>(())
		}));

		succ!(render!(mem::replace(&mut cx.cmp.visible, false)));
	}
}

impl Trigger {
	fn split_url(s: &str) -> Option<(UrlBuf, PathBufDyn)> {
		let (spec, path) = Spec::parse(s.as_bytes()).ok()?;
		if path.is_empty() && !AnyAsciiChar::SEP.predicate(s.bytes().last()?) {
			return None; // We don't complete a `sftp://test`, but `sftp://test/`
		}

		// Spec
		let spec = spec.zeroed();
		if spec.kind.is_local() && path.as_strand() == "~" {
			return None; // We don't complete a `~`, but `~/`
		}

		// Child
		let child = path.rsplit_pred(AnyAsciiChar::SEP).map_or(path.dyn_path(), |(_, c)| c).to_owned();

		// Parent
		let url = UrlCow::try_from((spec.clone().zeroed(), path)).ok()?;
		let abs = if let Some(u) = engine::try_absolute(&url) { u } else { url };
		let parent = abs.loc().try_strip_suffix(&child).ok()?;

		Some((clean_url(UrlCow::try_from((spec, parent)).ok()?), child))
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
		yazi_config::init_tests();
		yazi_fs::init();

		assert_eq!(Trigger::split_url(""), None);
		assert_eq!(Trigger::split_url("sftp://vps"), None);
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

		CWD.set(&"sftp://vps".parse::<UrlBuf>().unwrap(), || {});
		compare("sftp://vps/a", "sftp://vps/.", "a");
		compare("sftp://vps//a", "sftp://vps//", "a");
		compare("test-scope://aws/a", "test-scope://aws/.", "a");
		compare("test-scope://aws//a", "test-scope://aws//", "a");
	}

	#[cfg(windows)]
	#[test]
	fn test_split() {
		yazi_shared::init_tests();
		yazi_config::init_tests();
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
