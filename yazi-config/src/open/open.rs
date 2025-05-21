use std::{ops::Deref, path::Path};

use anyhow::Result;
use indexmap::IndexSet;
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_shared::MIME_DIR;

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
	pub fn all<'a, 'b, P, M>(&'a self, path: P, mime: M) -> impl Iterator<Item = &'a str> + 'b
	where
		'a: 'b,
		P: AsRef<Path> + 'b,
		M: AsRef<str> + 'b,
	{
		let is_dir = mime.as_ref() == MIME_DIR;
		self
			.rules
			.iter()
			.filter(move |&r| {
				r.mime.as_ref().is_some_and(|p| p.match_mime(&mime))
					|| r.name.as_ref().is_some_and(|p| p.match_path(&path, is_dir))
			})
			.flat_map(|r| &r.r#use)
			.map(String::as_str)
	}

	pub fn common<'a>(
		&'a self,
		targets: &[(impl AsRef<Path>, impl AsRef<str>)],
	) -> IndexSet<&'a str> {
		let each: Vec<IndexSet<&str>> = targets
			.iter()
			.map(|(p, m)| self.all(p, m).collect::<IndexSet<_>>())
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
