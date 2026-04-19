use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use indexmap::IndexMap;
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_fs::{File, cha::ChaType};
use yazi_shared::url::AsUrl;
use yazi_shim::toml::DeserializeOverHook;

use crate::{mix, open::{OpenRule, OpenRules}};

#[derive(Default, Deserialize, DeserializeOver2)]
pub struct Open {
	rules:         OpenRules,
	#[serde(default)]
	prepend_rules: Vec<OpenRule>,
	#[serde(default)]
	append_rules:  Vec<OpenRule>,
}

impl Deref for Open {
	type Target = OpenRules;

	fn deref(&self) -> &Self::Target { &self.rules }
}

impl Open {
	pub fn match_dummy<U, M>(&self, url: U, mime: M) -> Option<Arc<OpenRule>>
	where
		U: AsUrl,
		M: AsRef<str>,
	{
		let mime = mime.as_ref();
		let file = File::from_dummy(
			url.as_url().to_owned(),
			Some(if mime.starts_with("folder/") { ChaType::Dir } else { ChaType::File }),
		);

		self.matches(&file, mime)
	}

	pub fn match_common(&self, targets: &[(File, &str)]) -> impl Iterator<Item = Arc<OpenRule>> {
		let mut seen: IndexMap<Arc<OpenRule>, usize> = IndexMap::new();
		for (file, mime) in targets {
			if let Some(rule) = self.matches(file, mime) {
				*seen.entry(rule).or_default() += 1;
			}
		}

		seen.into_iter().filter(|&(_, count)| count == targets.len()).map(|(rule, _)| rule)
	}
}

impl DeserializeOverHook for Open {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		let rules: Vec<Arc<OpenRule>> =
			mix(self.prepend_rules, self.rules.unwrap_unchecked(), self.append_rules);

		Ok(Self { rules: rules.into(), ..Default::default() })
	}
}
