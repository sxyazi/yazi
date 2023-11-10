use std::mem;

use tokio::fs;
use yazi_config::keymap::{Exec, KeymapLayer};

use crate::{completion::Completion, emit};

pub struct Opt<'a> {
	before: &'a str,
	ticket: usize,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			before: e.named.get("before").map(|s| s.as_str()).unwrap_or_default(),
			ticket: e.named.get("ticket").and_then(|s| s.parse().ok()).unwrap_or(0),
		}
	}
}

impl Completion {
	#[inline]
	fn split_path(s: &str) -> (String, String) {
		match s.rsplit_once(|c| c == '/' || c == '\\') {
			Some((p, c)) => (format!("{p}/"), c.to_owned()),
			None => (".".to_owned(), s.to_owned()),
		}
	}

	pub fn trigger<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into() as Opt;
		if opt.ticket < self.ticket {
			return false;
		}

		self.ticket = opt.ticket;
		let (parent, child) = Self::split_path(opt.before);

		if self.caches.contains_key(&parent) {
			return self.show(
				&Exec::call("show", vec![])
					.with("cache-name", parent)
					.with("word", child)
					.with("ticket", opt.ticket),
			);
		}

		let ticket = self.ticket;
		tokio::spawn(async move {
			let mut dir = fs::read_dir(&parent).await?;
			let mut cache = Vec::new();
			while let Ok(Some(f)) = dir.next_entry().await {
				let Ok(meta) = f.metadata().await else {
					continue;
				};

				let sep = if !meta.is_dir() {
					""
				} else if cfg!(windows) {
					"\\"
				} else {
					"/"
				};
				cache.push(format!("{}{sep}", f.file_name().to_string_lossy()));
			}

			if !cache.is_empty() {
				emit!(Call(
					Exec::call("show", cache)
						.with("cache-name", parent)
						.with("word", child)
						.with("ticket", ticket)
						.vec(),
					KeymapLayer::Completion
				));
			}

			Ok::<(), anyhow::Error>(())
		});

		mem::replace(&mut self.visible, false)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_explode() {
		assert_eq!(Completion::split_path(""), (".".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path(" "), (".".to_owned(), " ".to_owned()));
		assert_eq!(Completion::split_path("/"), ("/".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("//"), ("//".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("/foo"), ("/".to_owned(), "foo".to_owned()));
		assert_eq!(Completion::split_path("/foo/"), ("/foo/".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("/foo/bar"), ("/foo/".to_owned(), "bar".to_owned()));

		// Windows
		assert_eq!(Completion::split_path("foo"), (".".to_owned(), "foo".to_owned()));
		assert_eq!(Completion::split_path("foo\\"), ("foo/".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("foo\\bar"), ("foo/".to_owned(), "bar".to_owned()));
		assert_eq!(Completion::split_path("foo\\bar\\"), ("foo\\bar/".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("C:\\"), ("C:/".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("C:\\foo"), ("C:/".to_owned(), "foo".to_owned()));
		assert_eq!(Completion::split_path("C:\\foo\\"), ("C:\\foo/".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("C:\\foo\\bar"), ("C:\\foo/".to_owned(), "bar".to_owned()));
	}
}
