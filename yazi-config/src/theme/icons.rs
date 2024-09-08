use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Deserializer};
use yazi_shared::{fs::File, theme::{Color, Icon, Style}, Condition};

use crate::{Pattern, Preset};

pub struct Icons {
	globs: Vec<(Pattern, Icon)>,
	dirs:  HashMap<String, Icon>,
	files: HashMap<String, Icon>,
	exts:  HashMap<String, Icon>,
	conds: Vec<(Condition, Icon)>,
}

impl Icons {
	pub fn matches(&self, file: &File) -> Option<&Icon> {
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
	fn match_by_glob(&self, file: &File) -> Option<&Icon> {
		self.globs.iter().find(|(p, _)| p.match_path(file.url(), file.is_dir())).map(|(_, i)| i)
	}

	#[inline]
	fn match_by_name(&self, file: &File) -> Option<&Icon> {
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
	fn match_by_ext(&self, file: &File) -> Option<&Icon> {
		let ext = file.url().extension()?.to_str()?;
		self.exts.get(ext).or_else(|| self.exts.get(&ext.to_ascii_lowercase()))
	}
}

impl<'de> Deserialize<'de> for Icons {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		pub struct Shadow {
			globs:         Vec<ShadowPat>,
			#[serde(default)]
			prepend_globs: Vec<ShadowPat>,
			#[serde(default)]
			append_globs:  Vec<ShadowPat>,

			dirs:         Vec<ShadowStr>,
			#[serde(default)]
			prepend_dirs: Vec<ShadowStr>,
			#[serde(default)]
			append_dirs:  Vec<ShadowStr>,

			files:         Vec<ShadowStr>,
			#[serde(default)]
			prepend_files: Vec<ShadowStr>,
			#[serde(default)]
			append_files:  Vec<ShadowStr>,

			exts:         Vec<ShadowStr>,
			#[serde(default)]
			prepend_exts: Vec<ShadowStr>,
			#[serde(default)]
			append_exts:  Vec<ShadowStr>,

			conds:         Vec<ShadowCond>,
			#[serde(default)]
			prepend_conds: Vec<ShadowCond>,
			#[serde(default)]
			append_conds:  Vec<ShadowCond>,
		}
		#[derive(Deserialize)]
		pub struct ShadowPat {
			name:     Pattern,
			text:     String,
			fg_dark:  Option<Color>,
			#[allow(dead_code)]
			fg_light: Option<Color>,
		}
		#[derive(Deserialize)]
		pub struct ShadowStr {
			name:     String,
			text:     String,
			fg_dark:  Option<Color>,
			#[allow(dead_code)]
			fg_light: Option<Color>,
		}
		#[derive(Deserialize)]
		pub struct ShadowCond {
			#[serde(rename = "if")]
			if_:      Condition,
			text:     String,
			fg_dark:  Option<Color>,
			#[allow(dead_code)]
			fg_light: Option<Color>,
		}

		let shadow = Shadow::deserialize(deserializer)?;

		let globs = Preset::mix(shadow.globs, shadow.prepend_globs, shadow.append_globs)
			.map(|v| {
				(v.name, Icon { text: v.text, style: Style { fg: v.fg_dark, ..Default::default() } })
			})
			.collect();

		let conds = Preset::mix(shadow.conds, shadow.prepend_conds, shadow.append_conds)
			.map(|v| {
				(v.if_, Icon { text: v.text, style: Style { fg: v.fg_dark, ..Default::default() } })
			})
			.collect();

		fn as_map(it: impl Iterator<Item = ShadowStr>) -> HashMap<String, Icon> {
			let mut map = HashMap::with_capacity(it.size_hint().0);
			for v in it {
				map
					.entry(v.name)
					.or_insert(Icon { text: v.text, style: Style { fg: v.fg_dark, ..Default::default() } });
			}
			map.shrink_to_fit();
			map
		}

		Ok(Self {
			globs,
			dirs: as_map(Preset::mix(shadow.dirs, shadow.prepend_dirs, shadow.append_dirs)),
			files: as_map(Preset::mix(shadow.files, shadow.prepend_files, shadow.append_files)),
			exts: as_map(Preset::mix(shadow.exts, shadow.prepend_exts, shadow.append_exts)),
			conds,
		})
	}
}
