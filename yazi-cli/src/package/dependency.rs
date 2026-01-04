use std::{io::BufWriter, path::{Path, PathBuf}, str::FromStr};

use anyhow::{Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use twox_hash::XxHash3_128;
use yazi_fs::Xdg;
use yazi_macro::ok_or_not_found;
use yazi_shared::BytesExt;

#[derive(Clone, Default)]
pub(crate) struct Dependency {
	pub(crate) r#use: String, // owner/repo:child
	pub(crate) name:  String, // child.yazi

	pub(crate) parent: String, // owner/repo
	pub(crate) child:  String, // child.yazi

	pub(crate) rev:  String,
	pub(crate) hash: String,

	pub(super) is_flavor: bool,
}

impl Dependency {
	pub(super) fn local(&self) -> PathBuf {
		Xdg::state_dir()
			.join("packages")
			.join(format!("{:x}", XxHash3_128::oneshot(self.remote().as_bytes())))
	}

	pub(super) fn remote(&self) -> String {
		// Support more Git hosting services in the future
		format!("https://github.com/{}.git", self.parent)
	}

	pub(super) fn target(&self) -> PathBuf {
		if self.is_flavor {
			Xdg::config_dir().join(format!("flavors/{}", self.name))
		} else {
			Xdg::config_dir().join(format!("plugins/{}", self.name))
		}
	}

	pub(super) fn identical(&self, other: &Self) -> bool {
		self.parent == other.parent && self.child == other.child
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

	pub(super) async fn plugin_files(dir: &Path) -> std::io::Result<Vec<String>> {
		let mut it = ok_or_not_found!(tokio::fs::read_dir(dir).await, return Ok(vec![]));
		let mut files: Vec<String> =
			["LICENSE", "README.md", "main.lua"].into_iter().map(Into::into).collect();
		while let Some(entry) = it.next_entry().await? {
			if let Ok(name) = entry.file_name().into_string()
				&& let Some(stripped) = name.strip_suffix(".lua")
				&& stripped != "main"
				&& stripped.as_bytes().kebab_cased()
			{
				files.push(name);
			}
		}
		files.sort_unstable();
		Ok(files)
	}

	pub(super) fn flavor_files() -> Vec<String> {
		["LICENSE", "LICENSE-tmtheme", "README.md", "flavor.toml", "preview.png", "tmtheme.xml"]
			.into_iter()
			.map(Into::into)
			.collect()
	}
}

impl FromStr for Dependency {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let mut parts = s.splitn(2, ':');

		let Some(parent) = parts.next() else { bail!("Package URL cannot be empty") };
		let child = parts.next().unwrap_or_default();

		let Some((_, repo)) = parent.split_once('/') else {
			bail!("Package URL `{parent}` must be in the format `owner/repository`")
		};

		let name = if child.is_empty() { repo } else { child };
		if !name.as_bytes().kebab_cased() {
			bail!("Package name `{name}` must be in kebab-case")
		}

		Ok(Self {
			r#use: s.to_owned(),
			name: format!("{name}.yazi"),
			parent: format!("{parent}{}", if child.is_empty() { ".yazi" } else { "" }),
			child: if child.is_empty() { String::new() } else { format!("{child}.yazi") },
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
			r#use: String,
			#[serde(default)]
			rev:   String,
			#[serde(default)]
			hash:  String,
		}

		let outer = Shadow::deserialize(deserializer)?;
		Ok(Self {
			rev: outer.rev,
			hash: outer.hash,
			..Self::from_str(&outer.r#use).map_err(serde::de::Error::custom)?
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
			r#use: &'a str,
			rev:   &'a str,
			hash:  &'a str,
		}

		Shadow { r#use: &self.r#use, rev: &self.rev, hash: &self.hash }.serialize(serializer)
	}
}
