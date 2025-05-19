use std::{borrow::Cow, ffi::OsString, mem, path::{MAIN_SEPARATOR_STR, Path, PathBuf}};

use tokio::fs;
use yazi_fs::{CWD, expand_path};
use yazi_macro::{emit, render};
use yazi_proxy::options::CmpItem;
use yazi_shared::{Id, event::{Cmd, CmdCow, Data}, natsort};

use crate::cmp::Cmp;

struct Opt {
	word:   Cow<'static, str>,
	ticket: Id,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			word:   c.take_first_str().unwrap_or_default(),
			ticket: c.get("ticket").and_then(Data::as_id).unwrap_or_default(),
		}
	}
}

impl Cmp {
	#[yazi_codegen::command]
	pub fn trigger(&mut self, opt: Opt) {
		if opt.ticket < self.ticket {
			return;
		}

		self.ticket = opt.ticket;
		let Some((parent, word)) = Self::split_path(&opt.word) else {
			return self.close(false);
		};

		if self.caches.contains_key(&parent) {
			return self.show(
				Cmd::default().with_any("cache-name", parent).with("word", word).with("ticket", opt.ticket),
			);
		}

		let ticket = self.ticket;
		tokio::spawn(async move {
			let mut dir = fs::read_dir(&parent).await?;
			let mut cache = vec![];

			// "/" is both a directory separator and the root directory per se
			// As there's no parent directory for the FS root, it is a special case
			if parent == Path::new("/") {
				cache.push(CmpItem { name: OsString::new(), is_dir: true });
			}

			while let Ok(Some(ent)) = dir.next_entry().await {
				if let Ok(ft) = ent.file_type().await {
					cache.push(CmpItem { name: ent.file_name(), is_dir: ft.is_dir() });
				}
			}

			if !cache.is_empty() {
				cache.sort_unstable_by(|a, b| {
					natsort(a.name.as_encoded_bytes(), b.name.as_encoded_bytes(), false)
				});
				emit!(Call(
					Cmd::new("cmp:show")
						.with_any("cache", cache)
						.with_any("cache-name", parent)
						.with("word", word)
						.with("ticket", ticket)
				));
			}

			Ok::<_, anyhow::Error>(())
		});

		render!(mem::replace(&mut self.visible, false));
	}

	fn split_path(s: &str) -> Option<(PathBuf, String)> {
		if s == "~" {
			return None; // We don't autocomplete a `~`, but `~/`
		}

		#[cfg(windows)]
		const SEP: [char; 2] = ['/', '\\'];
		#[cfg(not(windows))]
		const SEP: char = std::path::MAIN_SEPARATOR;

		Some(match s.rsplit_once(SEP) {
			Some(("", c)) => (PathBuf::from(MAIN_SEPARATOR_STR), c.to_owned()),
			Some((p, c)) => (expand_path(p), c.to_owned()),
			None => (CWD.load().to_path_buf(), s.to_owned()),
		})
	}
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;

	fn compare(s: &str, parent: &str, child: &str) -> bool {
		let (p, c) = Cmp::split_path(s).unwrap();
		let p = p.strip_prefix(yazi_fs::CWD.load().as_ref()).unwrap_or(&p);
		p == Path::new(parent) && c == child
	}

	#[cfg(unix)]
	#[test]
	fn test_split() {
		yazi_fs::init();
		assert!(compare("", "", ""));
		assert!(compare(" ", "", " "));
		assert!(compare("/", "/", ""));
		assert!(compare("//", "//", ""));
		assert!(compare("/foo", "/", "foo"));
		assert!(compare("/foo/", "/foo/", ""));
		assert!(compare("/foo/bar", "/foo/", "bar"));
	}

	#[cfg(windows)]
	#[test]
	fn test_split() {
		yazi_fs::init();
		assert!(compare("foo", "", "foo"));
		assert!(compare("foo\\", "foo\\", ""));
		assert!(compare("foo\\bar", "foo\\", "bar"));
		assert!(compare("foo\\bar\\", "foo\\bar\\", ""));
		assert!(compare("C:\\", "C:\\", ""));
		assert!(compare("C:\\foo", "C:\\", "foo"));
		assert!(compare("C:\\foo\\", "C:\\foo\\", ""));
		assert!(compare("C:\\foo\\bar", "C:\\foo\\", "bar"));
	}
}
