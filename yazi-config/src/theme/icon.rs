use anyhow::Result;
use hashbrown::HashMap;
use serde::Deserialize;
use yazi_codegen::{DeserializeOver2, Overlay};
use yazi_fs::File;
use yazi_shared::{NonEmptyString, url::UrlLike};
use yazi_shim::toml::DeserializeOverHook;

use super::{IconCond, IconConds, IconGlob, IconGlobs, IconNames, deserialize_icon_names};
use crate::{Icon as I, mix};

#[derive(Default, Deserialize, DeserializeOver2, Overlay)]
pub struct Icon {
	globs:         IconGlobs,
	#[serde(default)]
	prepend_globs: Vec<IconGlob>,
	#[serde(default)]
	append_globs:  Vec<IconGlob>,

	dirs:         IconNames,
	#[serde(default, deserialize_with = "deserialize_icon_names")]
	prepend_dirs: HashMap<NonEmptyString, I>,
	#[serde(default, deserialize_with = "deserialize_icon_names")]
	append_dirs:  HashMap<NonEmptyString, I>,

	files:         IconNames,
	#[serde(default, deserialize_with = "deserialize_icon_names")]
	prepend_files: HashMap<NonEmptyString, I>,
	#[serde(default, deserialize_with = "deserialize_icon_names")]
	append_files:  HashMap<NonEmptyString, I>,

	exts:         IconNames,
	#[serde(default, deserialize_with = "deserialize_icon_names")]
	prepend_exts: HashMap<NonEmptyString, I>,
	#[serde(default, deserialize_with = "deserialize_icon_names")]
	append_exts:  HashMap<NonEmptyString, I>,

	conds:         IconConds,
	#[serde(default)]
	prepend_conds: Vec<IconCond>,
	#[serde(default)]
	append_conds:  Vec<IconCond>,
}

impl Icon {
	pub fn matches(&self, file: &File, hovered: bool) -> Option<I> {
		if let Some(i) = self.globs.matches(file) {
			return Some(i);
		}

		let name = file.name().unwrap_or_default();
		match file.is_dir() {
			true if let Some(i) = self.dirs.matches(name) => Some(i),
			false if let Some(i) = self.files.matches(name) => Some(i),
			false if let Some(i) = self.exts.matches(file.url.ext().unwrap_or_default()) => Some(i),
			_ => self.conds.matches(file, hovered),
		}
	}
}

impl DeserializeOverHook for Icon {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		let dirs: HashMap<NonEmptyString, I> = self
			.append_dirs
			.into_iter()
			.chain(self.dirs.unwrap_unchecked())
			.chain(self.prepend_dirs)
			.collect();
		let files: HashMap<NonEmptyString, I> = self
			.append_files
			.into_iter()
			.chain(self.files.unwrap_unchecked())
			.chain(self.prepend_files)
			.collect();
		let exts: HashMap<NonEmptyString, I> = self
			.append_exts
			.into_iter()
			.chain(self.exts.unwrap_unchecked())
			.chain(self.prepend_exts)
			.collect();

		Ok(Self {
			globs: mix(self.prepend_globs, self.globs.unwrap_unchecked(), self.append_globs).into(),
			dirs: dirs.into(),
			files: files.into(),
			exts: exts.into(),
			conds: mix(self.prepend_conds, self.conds.unwrap_unchecked(), self.append_conds).into(),
			..Default::default()
		})
	}
}
