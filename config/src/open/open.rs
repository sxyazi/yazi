use std::{collections::BTreeMap, path::Path};

use indexmap::IndexSet;
use serde::{Deserialize, Deserializer};
use shared::MIME_DIR;

use super::Opener;
use crate::{Pattern, MERGED_YAZI};

#[derive(Debug)]
pub struct Open {
	openers: BTreeMap<String, IndexSet<Opener>>,
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
	pub fn openers<P, M>(&self, path: P, mime: M) -> Option<&IndexSet<Opener>>
	where
		P: AsRef<Path>,
		M: AsRef<str>,
	{
		self.rules.iter().find_map(|rule| {
			let is_folder = Some(mime.as_ref() == MIME_DIR);
			if rule.mime.as_ref().map_or(false, |m| m.matches(&mime))
				|| rule.name.as_ref().map_or(false, |n| n.match_path(&path, is_folder))
			{
				self.openers.get(&rule.use_)
			} else {
				None
			}
		})
	}

	#[inline]
	pub fn block_opener<P, M>(&self, path: P, mime: M) -> Option<&Opener>
	where
		P: AsRef<Path>,
		M: AsRef<str>,
	{
		self.openers(path, mime).and_then(|o| o.iter().find(|o| o.block))
	}

	pub fn common_openers(&self, targets: &[(impl AsRef<Path>, impl AsRef<str>)]) -> Vec<&Opener> {
		let grouped = targets.iter().filter_map(|(p, m)| self.openers(p, m)).collect::<Vec<_>>();
		let flat = grouped.iter().flat_map(|&g| g).collect::<IndexSet<_>>();
		flat.into_iter().filter(|&o| grouped.iter().all(|g| g.contains(o))).collect()
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
		let openers = outer.opener.into_iter().map(|(k, v)| (k, IndexSet::from_iter(v))).collect();
		Ok(Self { openers, rules: outer.open.rules })
	}
}
