use std::path::Path;

use serde::{Deserialize, Deserializer};
use yazi_shared::MIME_DIR;

use super::{Color, Style, StyleShadow};
use crate::Pattern;

pub struct Filetype {
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	pub typ:   Option<String>,
	pub style: Style,
}

impl Filetype {
	pub fn matches(&self, path: &Path, mime: Option<&str>) -> bool {
		let is_dir = mime == Some(MIME_DIR);
		let special_types = self.typ.as_ref().is_some_and(|t| match t.as_str() {
			"symlink" => path.is_symlink(),
			#[cfg(unix)]
			"executable" => {
				use std::os::unix::fs::PermissionsExt;
				let Ok(metadata) = path.metadata() else {
					return false;
				};

				let mode_bin = format!("{:b}", metadata.permissions().mode());
				return mode_bin.len() == 16 && match *mode_bin.as_bytes() {
					[.., _, _, a, _, _, b, _, _, c] => a == b'1' || b == b'1' || c == b'1',
					_ => false,
				}
			},
			_ => false
		});

		special_types
			|| self.name.as_ref().is_some_and(|n| n.match_path(path, is_dir))
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
			#[serde(rename = "type")]
			typ:  Option<String>,

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
					typ:   r.typ,
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
