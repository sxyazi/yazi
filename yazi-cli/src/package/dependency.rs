use std::{io::BufWriter, path::PathBuf, str::FromStr};

use anyhow::{Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use twox_hash::XxHash3_128;
use yazi_fs::Xdg;

#[derive(Default)]
pub(crate) struct Dependency {
	pub(crate) use_: String, // owner/repo:child
	pub(crate) name: String, // child.yazi

	pub(crate) parent: String, // owner/repo
	pub(crate) child:  String, // child

	pub(crate) rev:  String,
	pub(crate) hash: String,

	pub(super) is_flavor: bool,
}

impl Dependency {
	#[inline]
	pub(super) fn local(&self) -> PathBuf {
		Xdg::state_dir()
			.join("packages")
			.join(format!("{:x}", XxHash3_128::oneshot(self.remote().as_bytes())))
	}

	#[inline]
	pub(super) fn remote(&self) -> String {
		// Support more Git hosting services in the future
		format!("https://github.com/{}.git", self.parent)
	}

	pub(super) fn header(&self, s: &str) -> Result<()> {
		use crossterm::style::{Attribute, Print, SetAttributes};

		crossterm::execute!(
			BufWriter::new(std::io::stdout()),
			Print("\n"),
			SetAttributes(Attribute::Reverse.into()),
			SetAttributes(Attribute::Bold.into()),
			Print("  "),
			Print(s.replacen("{name}", &self.name, 1)),
			Print("  "),
			SetAttributes(Attribute::Reset.into()),
			Print("\n\n"),
		)?;
		Ok(())
	}
}

impl FromStr for Dependency {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let mut parts = s.splitn(2, ':');

		let Some(parent) = parts.next() else { bail!("Package url cannot be empty") };
		let child = parts.next().unwrap_or_default();

		let Some((_, repo)) = parent.split_once('/') else {
			bail!("Package url `{parent}` must be in the format `owner/repo`")
		};

		let name = if child.is_empty() { repo } else { child };
		if !name.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-')) {
			bail!("Package name `{name}` must be in kebab-case")
		}

		Ok(Self {
			use_: s.to_owned(),
			name: format!("{name}.yazi"),
			parent: format!("{parent}{}", if child.is_empty() { ".yazi" } else { "" }),
			child: child.to_owned(),
			..Default::default()
		})
	}
}

impl<'de> Deserialize<'de> for Dependency {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Shadow {
			#[serde(rename = "use")]
			use_: String,
			#[serde(default)]
			rev:  String,
			#[serde(default)]
			hash: String,
		}

		let outer = Shadow::deserialize(deserializer)?;
		Ok(Self {
			rev: outer.rev,
			hash: outer.hash,
			..Self::from_str(&outer.use_).map_err(serde::de::Error::custom)?
		})
	}
}

impl Serialize for Dependency {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		#[derive(Serialize)]
		struct Shadow<'a> {
			#[serde(rename = "use")]
			use_: &'a str,
			rev:  &'a str,
			hash: &'a str,
		}

		Shadow { use_: &self.use_, rev: &self.rev, hash: &self.hash }.serialize(serializer)
	}
}
