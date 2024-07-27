use std::{borrow::Cow, io::BufWriter, path::PathBuf};

use anyhow::Result;
use md5::{Digest, Md5};
use url::Url;
use yazi_shared::Xdg;

pub(crate) struct Package {
	pub(crate) repo:      String,
	pub(crate) child:     String,
	pub(crate) remote:    String,
	pub(crate) commit:    String,
	pub(super) is_flavor: bool,
}

impl Package {
	pub(super) fn new(url: &str, commit: Option<&str>) -> Self {
		match Url::parse(url) {
			Ok(mut url) => {
				let repo = url.path().trim_start_matches('/').to_string();
				let child = match url.fragment() {
					Some(fragment) => format!("{fragment}.yazi"),
					None => String::new(),
				};
				url.set_fragment(None);
				let remote = url.to_string();

				return Self {
					repo,
					child,
					remote,
					commit: commit.unwrap_or_default().to_owned(),
					is_flavor: false,
				};
			}
			Err(_) => {
				let mut parts = url.splitn(2, '#');

				let mut repo = parts.next().unwrap_or_default().to_owned();
				let child = if let Some(s) = parts.next() {
					format!("{s}.yazi")
				} else {
					repo.push_str(".yazi");
					String::new()
				};

				let remote = format!("https://github.com/{}.git", repo);

				return Self {
					repo,
					child,
					remote,
					commit: commit.unwrap_or_default().to_owned(),
					is_flavor: false,
				};
			}
		}
	}

	#[inline]
	pub(super) fn use_(&self) -> Cow<str> {
		if self.child.is_empty() {
			self.repo.trim_end_matches(".yazi").into()
		} else {
			format!("{}#{}", self.repo, self.child.trim_end_matches(".yazi")).into()
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
			.join(format!("{:x}", Md5::new_with_prefix(&self.remote).finalize()))
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
