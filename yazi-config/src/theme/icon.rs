use serde::{Deserialize, Deserializer};

use super::Style;
use crate::{theme::{Color, StyleShadow}, Pattern};

pub struct Icon {
	pub name:  Pattern,
	pub text:  String,
	pub style: Style,
}

impl Icon {
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Icon>, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct IconOuter {
			rules: Vec<IconRule>,
		}
		#[derive(Deserialize)]
		struct IconRule {
			name: Pattern,
			text: String,

			fg: Option<Color>,
		}

		Ok(
			IconOuter::deserialize(deserializer)?
				.rules
				.into_iter()
				.map(|r| Icon {
					name:  r.name,
					text:  r.text,
					style: StyleShadow { fg: r.fg, ..Default::default() }.into(),
				})
				.collect::<Vec<_>>(),
		)
	}
}
