use std::path::Path;

use serde::{Deserialize, Deserializer};

use super::{Color, Style, StyleShadow};
use crate::Pattern;

pub struct Filetype {
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	pub style: Style,
}

impl Filetype {
	pub fn matches(&self, path: &Path, mime: Option<impl AsRef<str>>, is_dir: bool) -> bool {
		if self.name.as_ref().is_some_and(|e| e.match_path(path, Some(is_dir))) {
			return true;
		}
		if let Some(mime) = mime {
			return self.mime.as_ref().is_some_and(|m| m.matches(mime));
		}
		false
	}
}

impl Filetype {
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Filetype>, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct FiletypeOuter {
			rules: Vec<FiletypeRule>,
		}
		#[derive(Deserialize)]
		struct FiletypeRule {
			name: Option<Pattern>,
			mime: Option<Pattern>,

			fg:          Option<Color>,
			bg:          Option<Color>,
			#[serde(default)]
			bold:        bool,
			#[serde(default)]
			dim:         bool,
			#[serde(default)]
			italic:      bool,
			#[serde(default)]
			underline:   bool,
			#[serde(default)]
			blink:       bool,
			#[serde(default)]
			blink_rapid: bool,
			#[serde(default)]
			reversed:    bool,
			#[serde(default)]
			hidden:      bool,
			#[serde(default)]
			crossed:     bool,
		}

		Ok(
			FiletypeOuter::deserialize(deserializer)?
				.rules
				.into_iter()
				.map(|r| Filetype {
					name:  r.name,
					mime:  r.mime,
					style: StyleShadow {
						fg:          r.fg,
						bg:          r.bg,
						bold:        r.bold,
						dim:         r.dim,
						italic:      r.italic,
						underline:   r.underline,
						blink:       r.blink,
						blink_rapid: r.blink_rapid,
						reversed:    r.reversed,
						hidden:      r.hidden,
						crossed:     r.crossed,
					}
					.into(),
				})
				.collect::<Vec<_>>(),
		)
	}
}
