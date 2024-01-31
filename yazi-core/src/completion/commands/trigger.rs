use std::{mem, path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR}};

use tokio::fs;
use yazi_shared::{emit, event::Cmd, render, Layer};

use crate::completion::Completion;

pub struct Opt {
	word:   String,
	ticket: usize,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			word:   c.take_first().unwrap_or_default(),
			ticket: c.take_name("ticket").and_then(|s| s.parse().ok()).unwrap_or(0),
		}
	}
}

impl Completion {
	#[inline]
	pub fn _trigger(word: &str, ticket: usize) {
		emit!(Call(
			Cmd::args("trigger", vec![word.to_owned()]).with("ticket", ticket),
			Layer::Completion
		));
	}

	pub fn trigger(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if opt.ticket < self.ticket {
			return;
		}

		self.ticket = opt.ticket;
		let (parent, child) = Self::split_path(&opt.word);

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
				let Ok(meta) = f.metadata().await else {
					continue;
				};

				cache.push(format!(
					"{}{}",
					f.file_name().to_string_lossy(),
					if meta.is_dir() { MAIN_SEPARATOR_STR } else { "" },
				));
			}

			if !cache.is_empty() {
				emit!(Call(
					Cmd::args("show", cache)
						.with("cache-name", parent)
						.with("word", child)
						.with("ticket", ticket),
					Layer::Completion
				));
			}

			Ok::<(), anyhow::Error>(())
		});

		render!(mem::replace(&mut self.visible, false));
	}

	#[inline]
	fn split_path(s: &str) -> (String, String) {
		match s.rsplit_once(MAIN_SEPARATOR) {
			Some((p, c)) => (format!("{p}{}", MAIN_SEPARATOR), c.to_owned()),
			None => (".".to_owned(), s.to_owned()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[cfg(unix)]
	#[test]
	fn test_explode() {
		assert_eq!(Completion::split_path(""), (".".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path(" "), (".".to_owned(), " ".to_owned()));
		assert_eq!(Completion::split_path("/"), ("/".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("//"), ("//".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("/foo"), ("/".to_owned(), "foo".to_owned()));
		assert_eq!(Completion::split_path("/foo/"), ("/foo/".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("/foo/bar"), ("/foo/".to_owned(), "bar".to_owned()));
	}

	#[cfg(windows)]
	#[test]
	fn test_explode() {
		assert_eq!(Completion::split_path("foo"), (".".to_owned(), "foo".to_owned()));
		assert_eq!(Completion::split_path("foo\\"), ("foo\\".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("foo\\bar"), ("foo\\".to_owned(), "bar".to_owned()));
		assert_eq!(Completion::split_path("foo\\bar\\"), ("foo\\bar\\".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("C:\\"), ("C:\\".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("C:\\foo"), ("C:\\".to_owned(), "foo".to_owned()));
		assert_eq!(Completion::split_path("C:\\foo\\"), ("C:\\foo\\".to_owned(), "".to_owned()));
		assert_eq!(Completion::split_path("C:\\foo\\bar"), ("C:\\foo\\".to_owned(), "bar".to_owned()));
	}
}
