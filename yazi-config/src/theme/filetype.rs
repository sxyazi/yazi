use std::path::Path;

use serde::{Deserialize, Deserializer};
use yazi_shared::MIME_DIR;

use super::{Color, Style, StyleShadow};
use crate::Pattern;

pub struct Filetype {
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	pub style: Style,
}

impl Filetype {
	pub fn matches(&self, path: &Path, mime: Option<&str>) -> bool {
		let is_dir = mime == Some(MIME_DIR);
		self.name.as_ref().is_some_and(|n| n.match_path(path, is_dir))
			|| self.mime.as_ref().zip(mime).map_or(false, |(m, s)| m.matches(s))
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
