use std::{collections::HashMap, ops::Deref};

use anyhow::Result;
use serde::{Deserialize, Deserializer};
use yazi_codegen::DeserializeOver2;
use yazi_fs::File;
use yazi_shared::Condition;

use crate::{Color, Icon as I, Pattern, Style};

#[derive(Default, Deserialize, DeserializeOver2)]
pub struct Icon {
	globs:         PatIcons,
	#[serde(default)]
	prepend_globs: PatIcons,
	#[serde(default)]
	append_globs:  PatIcons,

	dirs:         StrIcons,
	#[serde(default)]
	prepend_dirs: StrIcons,
	#[serde(default)]
	append_dirs:  StrIcons,

	files:         StrIcons,
	#[serde(default)]
	prepend_files: StrIcons,
	#[serde(default)]
	append_files:  StrIcons,

	exts:         StrIcons,
	#[serde(default)]
	prepend_exts: StrIcons,
	#[serde(default)]
	append_exts:  StrIcons,

	conds:         CondIcons,
	#[serde(default)]
	prepend_conds: CondIcons,
	#[serde(default)]
	append_conds:  CondIcons,
}

impl Icon {
	pub fn matches(&self, file: &File) -> Option<&I> {
		if let Some(i) = self.match_by_glob(file) {
			return Some(i);
		}

		if let Some(i) = self.match_by_name(file) {
			return Some(i);
		}

		let f = |s: &str| match s {
			"dir" => file.is_dir(),
			"hidden" => file.is_hidden(),
			"link" => file.is_link(),
			"orphan" => file.is_orphan(),
			"dummy" => file.is_dummy(),
			"block" => file.is_block(),
			"char" => file.is_char(),
			"fifo" => file.is_fifo(),
			"sock" => file.is_sock(),
			"exec" => file.is_exec(),
			"sticky" => file.is_sticky(),
			_ => false,
		};
		self.conds.iter().find(|(c, _)| c.eval(f) == Some(true)).map(|(_, i)| i)
	}

	#[inline]
	fn match_by_glob(&self, file: &File) -> Option<&I> {
		self.globs.iter().find(|(p, _)| p.match_path(&file.url, file.is_dir())).map(|(_, i)| i)
	}

	#[inline]
	fn match_by_name(&self, file: &File) -> Option<&I> {
		let name = file.name().to_str()?;
		if file.is_dir() {
			self.dirs.get(name).or_else(|| self.dirs.get(&name.to_ascii_lowercase()))
		} else {
			self
				.files
				.get(name)
				.or_else(|| self.files.get(&name.to_ascii_lowercase()))
				.or_else(|| self.match_by_ext(file))
		}
	}

	#[inline]
	fn match_by_ext(&self, file: &File) -> Option<&I> {
		let ext = file.url.extension()?.to_str()?;
		self.exts.get(ext).or_else(|| self.exts.get(&ext.to_ascii_lowercase()))
	}
}

impl Icon {
	pub(crate) fn reshape(self) -> Result<Self> {
		Ok(Self {
			globs: PatIcons(
				self.prepend_globs.0.into_iter().chain(self.globs.0).chain(self.append_globs.0).collect(),
			),
			dirs: StrIcons(
				self.append_dirs.0.into_iter().chain(self.dirs.0).chain(self.prepend_dirs.0).collect(),
			),
			files: StrIcons(
				self.append_files.0.into_iter().chain(self.files.0).chain(self.prepend_files.0).collect(),
			),
			exts: StrIcons(
				self.append_exts.0.into_iter().chain(self.exts.0).chain(self.prepend_exts.0).collect(),
			),
			conds: CondIcons(
				self.prepend_conds.0.into_iter().chain(self.conds.0).chain(self.append_conds.0).collect(),
			),
			..Default::default()
		})
	}
}

#[derive(Default)]
pub struct PatIcons(Vec<(Pattern, I)>);

impl Deref for PatIcons {
	type Target = Vec<(Pattern, I)>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'de> Deserialize<'de> for PatIcons {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Shadow {
			name: Pattern,
			text: String,
			fg:   Option<Color>,
		}

		Ok(Self(
			<Vec<Shadow>>::deserialize(deserializer)?
				.into_iter()
				.map(|s| (s.name, I { text: s.text, style: Style { fg: s.fg, ..Default::default() } }))
				.collect(),
		))
	}
}

#[derive(Default)]
pub struct StrIcons(HashMap<String, I>);

impl Deref for StrIcons {
	type Target = HashMap<String, I>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'de> Deserialize<'de> for StrIcons {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Shadow {
			name: String,
			text: String,
			fg:   Option<Color>,
		}

		Ok(Self(
			<Vec<Shadow>>::deserialize(deserializer)?
				.into_iter()
				.map(|s| (s.name, I { text: s.text, style: Style { fg: s.fg, ..Default::default() } }))
				.collect(),
		))
	}
}

#[derive(Default)]
pub struct CondIcons(Vec<(Condition, I)>);

impl Deref for CondIcons {
	type Target = Vec<(Condition, I)>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'de> Deserialize<'de> for CondIcons {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Shadow {
			r#if: Condition,
			text: String,
			fg:   Option<Color>,
		}

		Ok(Self(
			<Vec<Shadow>>::deserialize(deserializer)?
				.into_iter()
				.map(|s| (s.r#if, I { text: s.text, style: Style { fg: s.fg, ..Default::default() } }))
				.collect(),
		))
	}
}
