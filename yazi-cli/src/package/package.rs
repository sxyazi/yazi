use std::{borrow::Cow, io::BufWriter, path::PathBuf};

use anyhow::{Context, Result};
use md5::{Digest, Md5};
use yazi_shared::Xdg;

pub(crate) struct Package {
	pub(crate) host:      String,
	pub(crate) owner:     String,
	pub(crate) repo_name: String,
	pub(crate) child:     String,
	pub(crate) rev:       String,
	pub(super) is_flavor: bool,
}

impl Package {
	pub(super) fn new(url: &str, rev: Option<&str>) -> Result<Self> {
		let mut parts = url.splitn(2, ':');

		let mut repo_part = parts.next().unwrap_or_default().to_owned();
		let child = if let Some(s) = parts.next() {
			format!("{s}.yazi")
		} else {
			repo_part.push_str(".yazi");
			String::new()
		};

		let mut repo = repo_part.rsplit('/');
		let repo_name = repo.next().context("failed to get repo name")?.to_owned();
		let owner = repo.next().context("failed to get repo owner")?.to_owned();
		let host = repo.next().unwrap_or("github.com").to_owned();

		Ok(Self {
			repo_name,
			owner,
			host,
			child,
			rev: rev.unwrap_or_default().to_owned(),
			is_flavor: false,
		})
	}

	#[inline]
	pub(super) fn use_(&self) -> Cow<str> {
		if self.child.is_empty() {
			format!("{}/{}/{}", self.host, self.owner, self.repo_name.trim_end_matches(".yazi")).into()
		} else {
			format!(
				"{}/{}/{}:{}",
				self.host,
				self.owner,
				self.repo_name,
				self.child.trim_end_matches(".yazi")
			)
			.into()
		}
	}

	#[inline]
	pub(super) fn name(&self) -> &str {
		if self.child.is_empty() { self.repo_name.as_str() } else { self.child.as_str() }
	}

	#[inline]
	pub(super) fn local(&self) -> PathBuf {
		Xdg::state_dir()
			.join("packages")
			.join(format!("{:x}", Md5::new_with_prefix(self.remote()).finalize()))
	}

	#[inline]
	pub(super) fn remote(&self) -> String {
		format!("https://{}/{}/{}.git", self.host, self.owner, self.repo_name)
	}

	pub(super) fn header(&self, s: &str) -> Result<()> {
		use crossterm::style::{Attribute, Print, SetAttributes};

		crossterm::execute!(
			BufWriter::new(std::io::stdout()),
			Print("\n"),
			SetAttributes(Attribute::Reverse.into()),
			SetAttributes(Attribute::Bold.into()),
			Print("  "),
			Print(s.replacen("{name}", self.name(), 1)),
			Print("  "),
			SetAttributes(Attribute::Reset.into()),
			Print("\n\n"),
		)?;
		Ok(())
	}
}
