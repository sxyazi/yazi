use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Deserializer};
use yazi_shared::fs::File;

use super::{Color, Style, StyleShadow};
use crate::Pattern;

pub struct Filetype {
	pub is:    FiletypeIs,
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	pub style: Style,
}

impl Filetype {
	pub fn matches(&self, file: &File, mime: Option<&str>) -> bool {
		let b = match self.is {
			FiletypeIs::None => true,
			FiletypeIs::Block => file.cha.is_block_device(),
			FiletypeIs::Exec => file.cha.is_exec(),
			FiletypeIs::Fifo => file.cha.is_fifo(),
			FiletypeIs::Link => file.cha.is_link(),
			FiletypeIs::Orphan => file.cha.is_orphan(),
			FiletypeIs::Sock => file.cha.is_socket(),
			FiletypeIs::Sticky => file.cha.is_sticky(),
		};
		if !b {
			return false;
		}

		self.name.as_ref().is_some_and(|n| n.match_path(&file.url, file.is_dir()))
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
			#[serde(default)]
			is:   FiletypeIs,
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
					is:    r.is,
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

// --- FiletypeIs
#[derive(Default, Deserialize)]
#[serde(try_from = "String")]
pub enum FiletypeIs {
	#[default]
	None,
	Block,
	Exec,
	Fifo,
	Link,
	Orphan,
	Sock,
	Sticky,
}

impl FromStr for FiletypeIs {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"block" => Self::Block,
			"exec" => Self::Exec,
			"fifo" => Self::Fifo,
			"link" => Self::Link,
			"orphan" => Self::Orphan,
			"sock" => Self::Sock,
			"sticky" => Self::Sticky,
			_ => bail!("invalid filetype: {s}"),
		})
	}
}

impl TryFrom<String> for FiletypeIs {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::from_str(&s) }
}
