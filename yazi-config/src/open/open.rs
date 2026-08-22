use std::ops::Deref;

use anyhow::Result;
use indexmap::IndexSet;
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_fs::{cha::ChaType, file::File};
use yazi_shared::url::AsUrl;
use yazi_shim::toml::DeserializeOverHook;

use crate::{mix, open::{OpenRule, OpenRuleArc, OpenRules}};

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
	pub fn match_common(&self, targets: &[(File, &str)]) -> IndexSet<String> {
		let mut targets = targets.iter();
		let Some((file, mime)) = targets.next() else { return Default::default() };
		let Some(first) = self.matches(file, mime) else { return Default::default() };

		let mut common: IndexSet<_> = first.r#use.iter().cloned().collect();
		for (file, mime) in targets {
			let Some(rule) = self.matches(file, mime) else { return Default::default() };

			common.retain(|name| rule.r#use.contains(name));
			if common.is_empty() {
				break;
			}
		}
		common
	}

	pub fn match_dummy<U, M>(&self, url: U, mime: M) -> Option<OpenRuleArc>
	where
		U: AsUrl,
		M: AsRef<str>,
	{
		let mime = mime.as_ref();

		let is_dir = match mime.rsplit_once('/') {
			Some((_, last)) if last.is_empty() => false,
			Some(("folder", _)) => true,
			Some((rest, _)) => rest.ends_with("/folder"),
			None => false,
		};

		let file = File::from_dummy(
			url.as_url().to_owned(),
			Some(if is_dir { ChaType::Dir } else { ChaType::File }),
		);

		self.matches(&file, mime)
	}
}

impl DeserializeOverHook for Open {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		let rules: Vec<OpenRuleArc> =
			mix(self.prepend_rules, self.rules.unwrap_unchecked(), self.append_rules);

		Ok(Self { rules: rules.into(), ..Default::default() })
	}
}
