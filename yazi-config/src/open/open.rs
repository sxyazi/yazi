use std::ops::Deref;

use anyhow::Result;
use indexmap::IndexSet;
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_fs::{File, cha::ChaType};
use yazi_shared::url::AsUrl;
use yazi_shim::toml::DeserializeOverHook;

use crate::{Selectable, mix, open::OpenRule};

#[derive(Default, Deserialize, DeserializeOver2)]
pub struct Open {
	rules:         Vec<OpenRule>,
	#[serde(default)]
	prepend_rules: Vec<OpenRule>,
	#[serde(default)]
	append_rules:  Vec<OpenRule>,
}

impl Deref for Open {
	type Target = Vec<OpenRule>;

	fn deref(&self) -> &Self::Target { &self.rules }
}

impl Open {
	pub fn all<'a>(&'a self, file: &File, mime: &str) -> impl Iterator<Item = &'a str> {
		self
			.rules
			.iter()
			.find(move |&rule| rule.matches(file, mime))
			.into_iter()
			.flat_map(|r| &r.r#use)
			.map(String::as_str)
	}

	pub fn all_dummy<'a, U, M>(&'a self, url: U, mime: M) -> impl Iterator<Item = &'a str>
	where
		U: AsUrl,
		M: AsRef<str>,
	{
		let mime = mime.as_ref();
		let file = File::from_dummy(
			url.as_url().to_owned(),
			Some(if mime.starts_with("folder/") { ChaType::Dir } else { ChaType::File }),
		);

		self
			.rules
			.iter()
			.find(move |&rule| rule.matches(&file, mime))
			.into_iter()
			.flat_map(|r| &r.r#use)
			.map(String::as_str)
	}

	pub fn common<'a>(&'a self, targets: &[(File, &str)]) -> IndexSet<&'a str> {
		let each: Vec<IndexSet<&str>> = targets
			.iter()
			.map(|(file, mime)| self.all(file, mime).collect::<IndexSet<_>>())
			.filter(|s| !s.is_empty())
			.collect();

		let mut flat: IndexSet<_> = each.iter().flatten().copied().collect();
		flat.retain(|use_| each.iter().all(|e| e.contains(use_)));
		flat
	}
}

impl DeserializeOverHook for Open {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		Ok(Self { rules: mix(self.prepend_rules, self.rules, self.append_rules), ..Default::default() })
	}
}
