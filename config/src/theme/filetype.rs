use std::path::Path;

use serde::{Deserialize, Deserializer};

use super::Style;
use crate::{theme::Color, Pattern};

pub struct Filetype {
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	pub style: Style,
}

impl Filetype {
	pub fn matches(&self, path: &Path, mime: Option<impl AsRef<str>>, is_dir: bool) -> bool {
		if self.name.as_ref().map_or(false, |e| e.match_path(path, Some(is_dir))) {
			return true;
		}
		if let Some(mime) = mime {
			return self.mime.as_ref().map_or(false, |m| m.matches(mime));
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
			rules: Vec<FiletypeOuterStyle>,
		}
		#[derive(Deserialize)]
		struct FiletypeOuterStyle {
			name:      Option<Pattern>,
			mime:      Option<Pattern>,
			fg:        Option<Color>,
			bg:        Option<Color>,
			bold:      Option<bool>,
			underline: Option<bool>,
		}

		Ok(
			FiletypeOuter::deserialize(deserializer)?
				.rules
				.into_iter()
				.map(|r| Filetype {
					name:  r.name,
					mime:  r.mime,
					style: Style {
						fg:        r.fg,
						bg:        r.bg,
						bold:      r.bold,
						underline: r.underline,
					},
				})
				.collect::<Vec<_>>(),
		)
	}
}
