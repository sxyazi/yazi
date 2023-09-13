use std::{collections::BTreeMap, fmt, path::Path};

use indexmap::IndexSet;
use serde::{de::{self, Visitor}, Deserialize, Deserializer};
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
	#[serde(deserialize_with = "deserialize_from_str_or_vec")]
	use_: Vec<String>,
}

impl Default for Open {
	fn default() -> Self { toml::from_str(&MERGED_YAZI).unwrap() }
}

impl Open {
	pub fn openers<P, M>(&self, path: P, mime: M) -> Option<IndexSet<&Opener>>
	where
		P: AsRef<Path>,
		M: AsRef<str>,
	{
		self.rules.iter().find_map(|rule| {
			let is_folder = Some(mime.as_ref() == MIME_DIR);
			if rule.mime.as_ref().map_or(false, |m| m.matches(&mime))
				|| rule.name.as_ref().map_or(false, |n| n.match_path(&path, is_folder))
			{
				let openers = rule
					.use_
					.iter()
					.filter_map(|use_name| self.openers.get(use_name))
					.flatten()
					.collect::<IndexSet<_>>();

				if openers.is_empty() {
					return None;
				}

				Some(openers)
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
		self.openers(path, mime).and_then(|o| o.iter().find(|o| o.block).copied())
	}

	pub fn common_openers(&self, targets: &[(impl AsRef<Path>, impl AsRef<str>)]) -> Vec<&Opener> {
		let grouped = targets.iter().filter_map(|(p, m)| self.openers(p, m)).collect::<Vec<_>>();
		let flat = grouped.iter().flatten().collect::<IndexSet<_>>();
		flat.into_iter().filter(|&o| grouped.iter().all(|g| g.contains(o))).copied().collect()
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

fn deserialize_from_str_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
	D: Deserializer<'de>,
{
	struct StringVisitor;

	impl<'de> Visitor<'de> for StringVisitor {
		type Value = Vec<String>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a string, or array of strings")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: de::SeqAccess<'de>,
		{
			let mut strs = Vec::new();
			while let Some(value) = seq.next_element::<String>()? {
				strs.push(value);
			}
			Ok(strs)
		}

		fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
		where
			E: de::Error,
		{
			Ok(vec![value.to_owned()])
		}

		fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
		where
			E: de::Error,
		{
			Ok(vec![v])
		}
	}

	deserializer.deserialize_any(StringVisitor)
}
