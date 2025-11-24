use std::ops::Deref;

use anyhow::Result;
use indexmap::IndexSet;
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_shared::url::AsUrl;

use crate::{Preset, open::OpenRule};

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
	pub fn all<'a, 'b, U, M>(&'a self, url: U, mime: M) -> impl Iterator<Item = &'a str> + 'b
	where
		'a: 'b,
		U: AsUrl + 'b,
		M: AsRef<str> + 'b,
	{
		let is_dir = mime.as_ref().starts_with("folder/");
		self
			.rules
			.iter()
			.find(move |&r| {
				r.mime.as_ref().is_some_and(|p| p.match_mime(&mime))
					|| r.url.as_ref().is_some_and(|p| p.match_url(url.as_url(), is_dir))
			})
			.into_iter()
			.flat_map(|r| &r.r#use)
			.map(String::as_str)
	}

	pub fn common<'a, 'b, U, M>(&'a self, targets: &'b [(U, M)]) -> IndexSet<&'a str>
	where
		&'b U: AsUrl,
		M: AsRef<str>,
	{
		let each: Vec<IndexSet<&str>> = targets
			.iter()
			.map(|(u, m)| self.all(u, m).collect::<IndexSet<_>>())
			.filter(|s| !s.is_empty())
			.collect();

		let mut flat: IndexSet<_> = each.iter().flatten().copied().collect();
		flat.retain(|use_| each.iter().all(|e| e.contains(use_)));
		flat
	}
}

impl Open {
	pub(crate) fn reshape(self) -> Result<Self> {
		let any_file = self.append_rules.iter().any(|r| r.any_file());
		let any_dir = self.append_rules.iter().any(|r| r.any_dir());

		let it =
			self.rules.into_iter().filter(|r| !(any_file && r.any_file() || any_dir && r.any_dir()));

		Ok(Self {
			rules: Preset::mix(self.prepend_rules, it, self.append_rules).collect(),
			..Default::default()
		})
	}
}
