use anyhow::Result;
use hashbrown::HashMap;
use serde::{Deserialize, Deserializer};
use yazi_codegen::DeserializeOver2;
use yazi_fs::File;
use yazi_shared::{Condition, url::UrlLike};
use yazi_shim::toml::DeserializeOverHook;

use crate::{Icon as I, Mixable, Pattern, mix};

#[derive(Default, Deserialize, DeserializeOver2)]
pub struct Icon {
	globs:         Vec<IconPat>,
	#[serde(default)]
	prepend_globs: Vec<IconPat>,
	#[serde(default)]
	append_globs:  Vec<IconPat>,

	#[serde(deserialize_with = "deserialize_named_icons")]
	dirs:         HashMap<String, I>,
	#[serde(default, deserialize_with = "deserialize_named_icons")]
	prepend_dirs: HashMap<String, I>,
	#[serde(default, deserialize_with = "deserialize_named_icons")]
	append_dirs:  HashMap<String, I>,

	#[serde(deserialize_with = "deserialize_named_icons")]
	files:         HashMap<String, I>,
	#[serde(default, deserialize_with = "deserialize_named_icons")]
	prepend_files: HashMap<String, I>,
	#[serde(default, deserialize_with = "deserialize_named_icons")]
	append_files:  HashMap<String, I>,

	#[serde(deserialize_with = "deserialize_named_icons")]
	exts:         HashMap<String, I>,
	#[serde(default, deserialize_with = "deserialize_named_icons")]
	prepend_exts: HashMap<String, I>,
	#[serde(default, deserialize_with = "deserialize_named_icons")]
	append_exts:  HashMap<String, I>,

	conds:         Vec<IconCond>,
	#[serde(default)]
	prepend_conds: Vec<IconCond>,
	#[serde(default)]
	append_conds:  Vec<IconCond>,
}

impl Icon {
	pub fn matches(&self, file: &File, hovered: bool) -> Option<&I> {
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
			"hovered" => hovered,
			_ => false,
		};
		self.conds.iter().find(|&c| c.r#if.eval(f) == Some(true)).map(|c| &c.icon)
	}

	fn match_by_glob(&self, file: &File) -> Option<&I> {
		self.globs.iter().find(|&g| g.url.match_url(&file.url, file.is_dir())).map(|g| &g.icon)
	}

	fn match_by_name(&self, file: &File) -> Option<&I> {
		let name = file.name()?.to_str().ok()?;
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

	fn match_by_ext(&self, file: &File) -> Option<&I> {
		let ext = file.url.ext()?.to_str().ok()?;
		self.exts.get(ext).or_else(|| self.exts.get(&ext.to_ascii_lowercase()))
	}
}

impl DeserializeOverHook for Icon {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		Ok(Self {
			globs: mix(self.prepend_globs, self.globs, self.append_globs),
			dirs: self.append_dirs.into_iter().chain(self.dirs).chain(self.prepend_dirs).collect(),
			files: self.append_files.into_iter().chain(self.files).chain(self.prepend_files).collect(),
			exts: self.append_exts.into_iter().chain(self.exts).chain(self.prepend_exts).collect(),
			conds: mix(self.prepend_conds, self.conds, self.append_conds),
			..Default::default()
		})
	}
}

// --- IconPat
#[derive(Deserialize)]
pub struct IconPat {
	pub url:  Pattern,
	#[serde(flatten)]
	pub icon: I,
}

impl Mixable for IconPat {
	fn any_file(&self) -> bool { self.url.any_file() }

	fn any_dir(&self) -> bool { self.url.any_dir() }
}

// --- IconNamed
#[derive(Deserialize)]
struct IconNamed {
	name: String,
	#[serde(flatten)]
	icon: I,
}

fn deserialize_named_icons<'de, D>(deserializer: D) -> Result<HashMap<String, I>, D::Error>
where
	D: Deserializer<'de>,
{
	Ok(
		Vec::<IconNamed>::deserialize(deserializer)?
			.into_iter()
			.map(|entry| (entry.name, entry.icon))
			.collect(),
	)
}

// --- IconCond
#[derive(Deserialize)]
pub struct IconCond {
	pub r#if: Condition,
	#[serde(flatten)]
	pub icon: I,
}

impl Mixable for IconCond {}
