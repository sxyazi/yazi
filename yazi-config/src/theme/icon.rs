use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Deserializer};
use yazi_shared::fs::File;

use super::Style;
use crate::{preset::Preset, theme::{Color, StyleShadow}, Pattern};

pub struct Icon {
	pub is:    FiletypeIs,
	pub name:  Pattern,
	pub text:  String,
	pub style: Style,
}

impl Icon {
	pub fn matches(&self, file: &File) -> bool {
		let b = match self.is {
			FiletypeIs::None => true,
			FiletypeIs::Block => file.cha.is_block_device(),
			FiletypeIs::Char => file.cha.is_char_device(),
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

		self.name.match_path(&file.url, file.is_dir())
	}
}

impl Icon {
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Icon>, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct IconOuter {
			rules:         Vec<IconRule>,
			#[serde(default)]
			prepend_rules: Vec<IconRule>,
			#[serde(default)]
			append_rules:  Vec<IconRule>,
		}
		#[derive(Deserialize)]
		struct IconRule {
			#[serde(default)]
			is:   FiletypeIs,
			name: Pattern,
			text: String,

			fg: Option<Color>,
		}

		let mut outer = IconOuter::deserialize(deserializer)?;
		if outer.append_rules.iter().any(|r| r.name.any_file()) {
			outer.rules.retain(|r| !r.name.any_file());
		}
		if outer.append_rules.iter().any(|r| r.name.any_dir()) {
			outer.rules.retain(|r| !r.name.any_dir());
		}

		Preset::mix(&mut outer.rules, outer.prepend_rules, outer.append_rules);

		Ok(
			outer
				.rules
				.into_iter()
				.map(|r| Icon {
					is:    r.is,
					name:  r.name,
					text:  r.text,
					style: StyleShadow { fg: r.fg, ..Default::default() }.into(),
				})
				.collect(),
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
	Char,
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
			"char" => Self::Char,
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
