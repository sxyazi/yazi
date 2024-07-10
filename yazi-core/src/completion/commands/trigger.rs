use std::{borrow::Cow, mem, path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR}};

use tokio::fs;
use yazi_shared::{emit, event::{Cmd, Data}, render, Layer};

use crate::completion::Completion;

#[cfg(windows)]
const SEPARATOR: [char; 2] = ['/', '\\'];

#[cfg(not(windows))]
const SEPARATOR: char = std::path::MAIN_SEPARATOR;

pub struct Opt {
	word:   String,
	ticket: usize,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			word:   c.take_first_str().unwrap_or_default(),
			ticket: c.get("ticket").and_then(Data::as_usize).unwrap_or(0),
		}
	}
}

impl Completion {
	pub fn trigger(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if opt.ticket < self.ticket {
			return;
		}

		self.ticket = opt.ticket;
		let Some((parent, child)) = Self::split_path(&opt.word) else {
			return self.close(false);
		};

		if self.caches.contains_key(&parent) {
			return self.show(
				Cmd::new("show").with("cache-name", parent).with("word", child).with("ticket", opt.ticket),
			);
		}

		let ticket = self.ticket;
		tokio::spawn(async move {
			let mut dir = fs::read_dir(&parent).await?;
			let mut cache = vec![];
			while let Ok(Some(f)) = dir.next_entry().await {
				let Ok(meta) = f.metadata().await else { continue };

				cache.push(format!(
					"{}{}",
					f.file_name().to_string_lossy(),
					if meta.is_dir() { MAIN_SEPARATOR_STR } else { "" },
				));
			}

			if !cache.is_empty() {
				emit!(Call(
					Cmd::new("show")
						.with_any("cache", cache)
						.with("cache-name", parent)
						.with("word", child)
						.with("ticket", ticket),
					Layer::Completion
				));
			}

			Ok::<_, anyhow::Error>(())
		});

		render!(mem::replace(&mut self.visible, false));
	}

	fn split_path(s: &str) -> Option<(String, String)> {
		if s == "~" {
			return None; // We don't autocomplete a `~`, but `~/`
		}

		let s = if let Some(rest) = s.strip_prefix("~") {
			Cow::Owned(format!(
				"{}{rest}",
				dirs::home_dir().unwrap_or_default().to_string_lossy().trim_end_matches(SEPARATOR),
			))
		} else {
			Cow::Borrowed(s)
		};

		Some(match s.rsplit_once(SEPARATOR) {
			Some((p, c)) => (format!("{p}{}", MAIN_SEPARATOR), c.to_owned()),
			None => (".".to_owned(), s.into_owned()),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn compare(s: &str, parent: &str, child: &str) -> bool {
		matches!(Completion::split_path(s), Some((p, c)) if p == parent && c == child)
	}

	#[cfg(unix)]
	#[test]
	fn test_split() {
		assert!(compare("", ".", ""));
		assert!(compare(" ", ".", " "));
		assert!(compare("/", "/", ""));
		assert!(compare("//", "//", ""));
		assert!(compare("/foo", "/", "foo"));
		assert!(compare("/foo/", "/foo/", ""));
		assert!(compare("/foo/bar", "/foo/", "bar"));
	}

	#[cfg(windows)]
	#[test]
	fn test_split() {
		assert!(compare("foo", ".", "foo"));
		assert!(compare("foo\\", "foo\\", ""));
		assert!(compare("foo\\bar", "foo\\", "bar"));
		assert!(compare("foo\\bar\\", "foo\\bar\\", ""));
		assert!(compare("C:\\", "C:\\", ""));
		assert!(compare("C:\\foo", "C:\\", "foo"));
		assert!(compare("C:\\foo\\", "C:\\foo\\", ""));
		assert!(compare("C:\\foo\\bar", "C:\\foo\\", "bar"));
	}
}
