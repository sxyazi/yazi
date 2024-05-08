use std::{borrow::Cow, io::BufWriter, path::PathBuf};

use anyhow::Result;
use md5::{Digest, Md5};
use yazi_shared::Xdg;

#[derive(Debug, PartialEq)]
pub(crate) struct Package {
	pub(crate) repo:      String,
	/// When a package is a subdirectory of a repository, this field will be set
	/// to the subdirectory name.
	pub(crate) child:     String,
	pub(crate) commit:    String,
	pub(super) is_flavor: bool,
}

impl Package {
	pub(super) fn new(url: &str, commit: Option<&str>) -> Self {
		let mut parts = url.splitn(2, '#');

		let repo = parts.next().unwrap_or_default().to_owned();
		let child = parts.next().unwrap_or_default().to_owned();

		Self { repo, child, commit: commit.unwrap_or_default().to_owned(), is_flavor: false }
	}

	#[inline]
	pub(super) fn use_(&self) -> Cow<str> {
		if self.child.is_empty() {
			self.repo.clone().into()
		} else {
			format!("{}#{}", self.repo, self.child).into()
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

	pub(super) fn output(&self, s: &str) -> Result<()> {
		use crossterm::style::{Attribute, Print, SetAttributes};

		crossterm::execute!(
			BufWriter::new(std::io::stdout()),
			Print("\n"),
			SetAttributes(Attribute::Reverse.into()),
			SetAttributes(Attribute::Bold.into()),
			Print("  "),
			Print(s.replacen("{name}", self.name().unwrap_or_default(), 1)),
			Print("  "),
			SetAttributes(Attribute::NoBold.into()),
			SetAttributes(Attribute::NoReverse.into()),
			Print("\n\n"),
		)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_adds_yazi_suffix() {
		// if a package doesn't have a suffix, it should add ".yazi" to the end
		let package = Package::new("abcd/my-plugin.yazi", None);
		assert_eq!(package, Package {
			repo:      "abcd/my-plugin.yazi".to_owned(),
			child:     "".to_owned(),
			commit:    "".to_owned(),
			is_flavor: false,
		});

		assert_eq!(package.use_(), "abcd/my-plugin.yazi");
		assert_eq!(package.name(), Some("my-plugin.yazi"));
		assert_eq!(package.remote(), "https://github.com/abcd/my-plugin.yazi.git");
	}

	#[test]
	fn test_supports_subdirectories() {
		let package = Package::new("yazi-rs/flavors#catppuccin-mocha.yazi", None);
		assert_eq!(package, Package {
			repo:      "yazi-rs/flavors".to_owned(),
			child:     "catppuccin-mocha.yazi".to_owned(),
			commit:    "".to_owned(),
			is_flavor: false,
		});

		assert_eq!(package.use_(), "yazi-rs/flavors#catppuccin-mocha.yazi");
		assert_eq!(package.name(), Some("catppuccin-mocha.yazi"));
		assert_eq!(package.remote(), "https://github.com/yazi-rs/flavors.git");
	}

	#[test]
	fn test_github_with_yazi_suffix() {
		// when a github repository ends with .yazi, it should not add another .yazi
		let package = Package::new("DreamMaoMao/keyjump.yazi", None);
		assert_eq!(package, Package {
			repo:      "DreamMaoMao/keyjump.yazi".to_owned(),
			child:     "".to_owned(),
			commit:    "".to_owned(),
			is_flavor: false,
		});

		assert_eq!(package.use_(), "DreamMaoMao/keyjump.yazi");
		assert_eq!(package.name(), Some("keyjump.yazi"));
		assert_eq!(package.remote(), "https://github.com/DreamMaoMao/keyjump.yazi.git");
	}
}
