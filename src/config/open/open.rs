use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Deserializer};
use xdg::BaseDirectories;

use super::Opener;
use crate::config::Pattern;

#[derive(Debug)]
pub struct Open {
	openers: BTreeMap<String, Vec<Opener>>,
	rules:   Vec<OpenRule>,
}

#[derive(Debug, Deserialize)]
struct OpenRule {
	name: Option<Pattern>,
	mime: Option<Pattern>,
	#[serde(rename = "use")]
	use_: String,
}

impl Open {
	pub fn new() -> Self {
		let path = BaseDirectories::new().unwrap().get_config_file("yazi/yazi.toml");
		toml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
	}

	pub fn opener(&self, path: &Path, mime: &str) -> Option<&Opener> {
		self.rules.iter().find_map(|rule| {
			if rule.name.as_ref().map_or(false, |e| e.match_path(path, Some(false)))
				|| rule.mime.as_ref().map_or(false, |m| m.matches(mime))
			{
				self.openers.get(&rule.use_).and_then(|v| v.first())
			} else {
				None
			}
		})
	}
}

impl<'de> Deserialize<'de> for Open {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Outer {
			opener: BTreeMap<String, Vec<Opener>>,
			open:   OuterOpen,
		}
		#[derive(Deserialize)]
		struct OuterOpen {
			rules: Vec<OpenRule>,
		}

		let outer = Outer::deserialize(deserializer)?;
		Ok(Self { openers: outer.opener, rules: outer.open.rules })
	}
}
