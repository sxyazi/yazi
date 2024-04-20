use serde::{Deserialize, Deserializer};
use yazi_shared::{fs::File, theme::{Color, Style, StyleShadow}};

use crate::{preset::Preset, theme::Is, Pattern};

pub struct Icon {
	is:        Is,
	name:      Pattern,
	pub text:  String,
	pub style: Style,
}

impl Icon {
	pub fn matches(&self, file: &File) -> bool {
		if !self.is.check(&file.cha) {
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
			is:   Is,
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
