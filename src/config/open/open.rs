use std::{collections::{BTreeMap, BTreeSet}, path::Path};

use serde::{Deserialize, Deserializer};

use super::Opener;
use crate::config::{Pattern, MERGED_YAZI};

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
	pub fn new() -> Self { toml::from_str(&MERGED_YAZI).unwrap() }

	pub fn openers(&self, path: impl AsRef<Path>, mime: impl AsRef<str>) -> Option<Vec<&Opener>> {
		self.rules.iter().find_map(|rule| {
			if rule.name.as_ref().map_or(false, |e| e.match_path(&path, Some(false)))
				|| rule.mime.as_ref().map_or(false, |m| m.matches(&mime))
			{
				self.openers.get(&rule.use_).map(|v| v.iter().collect())
			} else {
				None
			}
		})
	}

	pub fn common_openers<'a>(
		&self,
		targets: &[(impl AsRef<Path>, impl AsRef<str>)],
	) -> Vec<&Opener> {
		let grouped = targets.into_iter().filter_map(|(p, m)| self.openers(p, m)).collect::<Vec<_>>();
		let flat = grouped.iter().flatten().cloned().collect::<BTreeSet<_>>();
		flat.into_iter().filter(|o| grouped.iter().all(|g| g.contains(o))).collect()
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
