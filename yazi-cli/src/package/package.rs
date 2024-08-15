use std::{borrow::Cow, io::BufWriter, path::PathBuf};

use anyhow::Result;
use md5::{Digest, Md5};
use yazi_shared::Xdg;

pub(crate) struct Package {
	pub(crate) repo:      String,
	pub(crate) child:     String,
	pub(crate) rev:       String,
	pub(super) is_flavor: bool,
}

impl Package {
	pub(super) fn new(url: &str, rev: Option<&str>) -> Self {
		let mut parts = url.splitn(2, ':');

		let mut repo = parts.next().unwrap_or_default().to_owned();
		let child = if let Some(s) = parts.next() {
			format!("{s}.yazi")
		} else {
			repo.push_str(".yazi");
			String::new()
		};

		Self { repo, child, rev: rev.unwrap_or_default().to_owned(), is_flavor: false }
	}

	#[inline]
	pub(super) fn use_(&self) -> Cow<str> {
		if self.child.is_empty() {
			self.repo.trim_end_matches(".yazi").into()
		} else {
			format!("{}:{}", self.repo, self.child.trim_end_matches(".yazi")).into()
		}
	}

	#[inline]
	pub(super) fn name(&self) -> Option<&str> {
		let s = if self.child.is_empty() {
			self.repo.split('/').last().filter(|s| !s.is_empty())
		} else {
			Some(self.child.as_str())
		};

		s.filter(|s| s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-' | b'.')))
	}

	#[inline]
	pub(super) fn local(&self) -> PathBuf {
		Xdg::state_dir()
			.join("packages")
			.join(format!("{:x}", Md5::new_with_prefix(self.remote()).finalize()))
	}

	#[inline]
	pub(super) fn remote(&self) -> String {
		// Support more Git hosting services in the future
		format!("https://github.com/{}.git", self.repo)
	}

	pub(super) fn header(&self, s: &str) -> Result<()> {
		use crossterm::style::{Attribute, Print, SetAttributes};

		crossterm::execute!(
			BufWriter::new(std::io::stdout()),
			Print("\n"),
			SetAttributes(Attribute::Reverse.into()),
			SetAttributes(Attribute::Bold.into()),
			Print("  "),
			Print(s.replacen("{name}", self.name().unwrap_or_default(), 1)),
			Print("  "),
			SetAttributes(Attribute::Reset.into()),
			Print("\n\n"),
		)?;
		Ok(())
	}
}
