use std::{ffi::OsString, mem, path::MAIN_SEPARATOR_STR};

use anyhow::Result;
use yazi_fs::{CWD, path::expand_url, provider::{self, DirReader, FileHolder}};
use yazi_macro::{act, render, succ};
use yazi_parser::cmp::{CmpItem, ShowOpt, TriggerOpt};
use yazi_proxy::CmpProxy;
use yazi_shared::{OsStrSplit, event::Data, natsort, url::{UrlBuf, UrlCow, UrnBuf}};

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
			return act!(cmp:show, cx, ShowOpt { cache_name: parent, word, ticket, ..Default::default() });
		}

		let ticket = cmp.ticket;
		tokio::spawn(async move {
			let mut dir = provider::read_dir(&parent).await?;
			let mut cache = vec![];

			// "/" is both a directory separator and the root directory per se
			// As there's no parent directory for the FS root, it is a special case
			if parent.loc.as_os_str() == "/" {
				cache.push(CmpItem { name: OsString::new(), is_dir: true });
			}

			while let Ok(Some(ent)) = dir.next().await {
				if let Ok(ft) = ent.file_type().await {
					cache.push(CmpItem { name: ent.name().into_owned(), is_dir: ft.is_dir() });
				}
			}

			if !cache.is_empty() {
				cache.sort_unstable_by(|a, b| {
					natsort(a.name.as_encoded_bytes(), b.name.as_encoded_bytes(), false)
				});
				CmpProxy::show(ShowOpt { cache, cache_name: parent, word, ticket });
			}

			Ok::<_, anyhow::Error>(())
		});

		succ!(render!(mem::replace(&mut cmp.visible, false)));
	}
}

impl Trigger {
	fn split_url(s: &str) -> Option<(UrlBuf, UrnBuf)> {
		let (scheme, path, ..) = UrlCow::parse(s.as_bytes()).ok()?;

		if !scheme.is_virtual() && path.as_os_str() == "~" {
			return None; // We don't autocomplete a `~`, but `~/`
		}

		#[cfg(windows)]
		const SEP: &[char] = &['/', '\\'];
		#[cfg(not(windows))]
		const SEP: char = std::path::MAIN_SEPARATOR;

		Some(match path.as_os_str().rsplit_once(SEP) {
			Some((p, c)) if p.is_empty() => {
				(UrlBuf { loc: MAIN_SEPARATOR_STR.into(), scheme: scheme.into() }, c.into())
			}
			Some((p, c)) => (expand_url(UrlBuf { loc: p.into(), scheme: scheme.into() }), c.into()),
			None => (CWD.load().as_ref().clone(), path.into()),
		})
	}
}

#[cfg(test)]
mod tests {
	use yazi_shared::url::Urn;

	use super::*;

	fn compare(s: &str, parent: &str, child: &str) {
		let (mut p, c) = Trigger::split_url(s).unwrap();
		if let Some(u) = p.strip_prefix(yazi_fs::CWD.load().as_ref()) {
			p = UrlBuf::from(&**u);
		}
		assert_eq!((p, c.as_urn()), (parent.parse().unwrap(), Urn::new(child)));
	}

	#[cfg(unix)]
	#[test]
	fn test_split() {
		yazi_fs::init();
		compare("", "", "");
		compare(" ", "", " ");
		compare("/", "/", "");
		compare("//", "//", "");
		compare("/foo", "/", "foo");
		compare("/foo/", "/foo/", "");
		compare("/foo/bar", "/foo/", "bar");
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
