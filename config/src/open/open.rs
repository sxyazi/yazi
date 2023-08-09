use std::{collections::{BTreeMap, BTreeSet}, path::Path};

use serde::{Deserialize, Deserializer};
use shared::MIME_DIR;

use super::Opener;
use crate::{Pattern, MERGED_YAZI};

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

impl Default for Open {
	fn default() -> Self { toml::from_str(&MERGED_YAZI).unwrap() }
}

impl Open {
	pub fn openers<P, M>(&self, path: P, mime: M) -> Option<Vec<&Opener>>
	where
		P: AsRef<Path>,
		M: AsRef<str>,
	{
		self.rules.iter().find_map(|rule| {
			if rule.name.as_ref().map_or(false, |e| e.match_path(&path, Some(mime.as_ref() == MIME_DIR)))
				|| rule.mime.as_ref().map_or(false, |m| m.matches(&mime))
			{
				self.openers.get(&rule.use_).map(|v| v.iter().collect())
			} else {
				None
			}
		})
	}

	pub fn common_openers(&self, targets: &[(impl AsRef<Path>, impl AsRef<str>)]) -> Vec<&Opener> {
		let grouped = targets.iter().filter_map(|(p, m)| self.openers(p, m)).collect::<Vec<_>>();
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
