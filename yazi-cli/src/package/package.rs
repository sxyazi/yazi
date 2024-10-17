use std::{borrow::Cow, io::BufWriter, path::PathBuf, sync::OnceLock};

use anyhow::{Error, Result};
use md5::{Digest, Md5};
use regex::Regex;
use yazi_shared::Xdg;

static PACKAGE_RAW_URL_RE: OnceLock<Regex> = OnceLock::new();
static PACKAGE_SHORT_URL_RE: OnceLock<Regex> = OnceLock::new();

#[inline]
fn package_raw_url_re() -> &'static Regex {
	PACKAGE_RAW_URL_RE.get_or_init(|| {
		Regex::new(r"^(?P<proto>[^:]+)://(?P<host>[^/]+)/(?P<url_path>[^:]+)(:(?P<child>.*))?$")
			.unwrap()
	})
}

#[inline]
fn package_short_url_re() -> &'static Regex {
	PACKAGE_SHORT_URL_RE.get_or_init(|| {
		Regex::new(r"^((?P<host>[^/]+)/)?(?P<owner>[^/]+)/(?P<repo>[^:]+)(:(?P<child>.*))?$").unwrap()
	})
}

#[derive(Debug)]
pub(crate) struct Package {
	pub(crate) proto:     String,
	pub(crate) host:      String,
	pub(crate) url_path:  String,
	pub(crate) child:     String,
	pub(crate) rev:       String,
	pub(super) is_flavor: bool,
}

impl Package {
	pub(super) fn new(url: &str, rev: Option<&str>) -> Result<Self> {
		let rev = rev.unwrap_or_default().to_owned();
		let is_flavor = false;

		if let Some(raw_url_match) = package_raw_url_re().captures(url) {
			let proto = raw_url_match["proto"].to_owned();
			let host = raw_url_match["host"].to_owned();
			let mut url_path = raw_url_match["url_path"].to_owned();
			let child = if let Some(child) = raw_url_match.name("child") {
				format!("{}.yazi", child.as_str())
			} else {
				url_path.push_str(".yazi");
				String::new()
			};

			Ok(Self { proto, host, url_path, child, rev, is_flavor })
		} else if let Some(short_url_match) = package_short_url_re().captures(url) {
			let proto = "https".to_owned();
			let host =
				short_url_match.name("host").map(|m| m.as_str()).unwrap_or("github.com").to_owned();
			let owner = &short_url_match["owner"];
			let repo = &short_url_match["repo"];
			let mut url_path = format!("{owner}/{repo}");

			let child = if let Some(child) = short_url_match.name("child") {
				format!("{}.yazi", child.as_str())
			} else {
				url_path.push_str(".yazi");
				String::new()
			};

			Ok(Self { proto, host, url_path, child, rev, is_flavor })
		} else {
			Err(Error::msg("invalid package url"))
		}
	}

	#[inline]
	pub(super) fn use_(&self) -> Cow<str> {
		if self.child.is_empty() {
			format!("{}://{}/{}", self.proto, self.host, self.url_path.trim_end_matches(".yazi")).into()
		} else {
			format!(
				"{}://{}/{}:{}",
				self.proto,
				self.host,
				self.url_path,
				self.child.trim_end_matches(".yazi")
			)
			.into()
		}
	}

	#[inline]
	pub(super) fn name(&self) -> &str {
		if self.child.is_empty() {
			self.url_path.rsplit('/').next().unwrap_or(&self.url_path)
		} else {
			self.child.as_str()
		}
	}

	#[inline]
	pub(super) fn local(&self) -> PathBuf {
		Xdg::state_dir()
			.join("packages")
			.join(format!("{:x}", Md5::new_with_prefix(self.remote()).finalize()))
	}

	#[inline]
	pub(super) fn remote(&self) -> String {
		format!("{}://{}/{}", self.proto, self.host, self.url_path)
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn two_component_short_url() -> Result<()> {
		let url = "owner/repo";

		let pkg = Package::new(url, None)?;

		assert_eq!(pkg.proto, "https");
		assert_eq!(pkg.host, "github.com");
		assert_eq!(pkg.url_path, "owner/repo.yazi");
		assert_eq!(pkg.child, "");

		assert_eq!(pkg.remote(), "https://github.com/owner/repo.yazi");

		Ok(())
	}

	#[test]
	fn three_component_short_url() -> Result<()> {
		let url = "codeberg.org/owner/repo";

		let pkg = Package::new(url, None)?;

		assert_eq!(pkg.proto, "https");
		assert_eq!(pkg.host, "codeberg.org");
		assert_eq!(pkg.url_path, "owner/repo.yazi");
		assert_eq!(pkg.child, "");

		assert_eq!(pkg.remote(), "https://codeberg.org/owner/repo.yazi");

		Ok(())
	}

	#[test]
	fn two_component_short_url_with_child_path() -> Result<()> {
		let url = "owner/repo:my-plugin";

		let pkg = Package::new(url, None)?;

		assert_eq!(pkg.proto, "https");
		assert_eq!(pkg.host, "github.com");
		assert_eq!(pkg.url_path, "owner/repo");
		assert_eq!(pkg.child, "my-plugin.yazi");

		assert_eq!(pkg.remote(), "https://github.com/owner/repo");
		assert_eq!(pkg.use_(), "https://github.com/owner/repo:my-plugin");

		Ok(())
	}

	#[test]
	fn raw_ssh_url() -> Result<()> {
		let url = "ssh://git@my-host:6969/my-plugin";

		let pkg = Package::new(url, None)?;

		assert_eq!(pkg.proto, "ssh");
		assert_eq!(pkg.host, "git@my-host:6969");
		assert_eq!(pkg.url_path, "my-plugin.yazi");
		assert_eq!(pkg.child, "");

		assert_eq!(pkg.remote(), "ssh://git@my-host:6969/my-plugin.yazi");

		Ok(())
	}

	#[test]
	fn raw_ssh_url_with_child_path() -> Result<()> {
		let url = "ssh://git@192.168.0.69:2222/~/my-repo.git:my-plugin";

		let pkg = Package::new(url, None)?;

		assert_eq!(pkg.proto, "ssh");
		assert_eq!(pkg.host, "git@192.168.0.69:2222");
		assert_eq!(pkg.url_path, "~/my-repo.git");
		assert_eq!(pkg.child, "my-plugin.yazi");

		assert_eq!(pkg.remote(), "ssh://git@192.168.0.69:2222/~/my-repo.git");
		assert_eq!(pkg.use_(), "ssh://git@192.168.0.69:2222/~/my-repo.git:my-plugin");

		Ok(())
	}

	#[test]
	fn raw_http_url_with_non_standard_path() -> Result<()> {
		let url = "https://example.com/xxx/yyy/zzz/owner/repo:my-plugin";

		let pkg = Package::new(url, None)?;

		assert_eq!(pkg.proto, "https");
		assert_eq!(pkg.host, "example.com");
		assert_eq!(pkg.url_path, "xxx/yyy/zzz/owner/repo");
		assert_eq!(pkg.child, "my-plugin.yazi");

		assert_eq!(pkg.remote(), "https://example.com/xxx/yyy/zzz/owner/repo");
		assert_eq!(pkg.use_(), "https://example.com/xxx/yyy/zzz/owner/repo:my-plugin");

		Ok(())
	}

	#[test]
	fn invalid_url() {
		let url = "one-component-url???";

		let pkg = Package::new(url, None);

		assert!(pkg.is_err());
	}
}
