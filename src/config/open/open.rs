use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;
use xdg::BaseDirectories;

use crate::config::Pattern;

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Opener {
	pub cmd:    String,
	pub args:   Vec<String>,
	#[serde(default)]
	pub block:  bool,
	#[serde(skip)]
	pub spread: bool,
}

#[derive(Deserialize, Debug)]
pub struct Open {
	#[serde(skip)]
	openers: BTreeMap<String, Vec<Opener>>,

	rules: Vec<OpenRule>,
}

#[derive(Deserialize, Debug)]
struct OpenRule {
	name: Option<Pattern>,
	mime: Option<Pattern>,
	#[serde(rename = "use")]
	use_: String,
}

impl Open {
	pub fn new() -> Open {
		#[derive(Deserialize)]
		struct Outer {
			opener: BTreeMap<String, Vec<Opener>>,
			open:   Open,
		}

		let path = BaseDirectories::new().unwrap().get_config_file("yazi/yazi.toml");
		let mut outer = toml::from_str::<Outer>(&fs::read_to_string(path).unwrap()).unwrap();

		for opener in outer.opener.values_mut() {
			for one in opener.iter_mut() {
				one.spread = one.args.iter().any(|a| a == "$*");
			}
		}

		Self { openers: outer.opener, rules: outer.open.rules }
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
