use std::{path::Path, str::FromStr};

use serde::{Deserialize, Deserializer};
use yazi_shared::MIME_DIR;

use super::{Color, Style, StyleShadow};
use crate::Pattern;

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub enum FileKind {
	Executable,
	Symlink,
}

impl FromStr for FileKind {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"executable" => Ok(Self::Executable),
			"symlink" => Ok(Self::Symlink),
			_ => Err(anyhow::anyhow!("invalid file kind"))
		}
	}
}

impl TryFrom<String> for FileKind {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::from_str(&s) }
}

pub struct Filetype {
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	pub kind:  Option<FileKind>,
	pub style: Style,
}

impl Filetype {
	pub fn matches(&self, path: &Path, mime: Option<&str>) -> bool {
		let is_dir = mime == Some(MIME_DIR);
		let kind_check = self.kind.as_ref().is_some_and(|t| match t {
			FileKind::Symlink => path.is_symlink(),
			#[cfg(unix)]
			FileKind::Executable => {
				use std::os::unix::fs::PermissionsExt;
				let Ok(metadata) = path.metadata() else {
					return false;
				};

				metadata.permissions().mode() & 0o100111 > 0o100000
			},
		});

		kind_check
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
			kind: Option<FileKind>,

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
					kind:  r.kind,
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
