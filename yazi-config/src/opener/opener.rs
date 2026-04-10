use std::{mem, ops::Deref};

use anyhow::Result;
use hashbrown::HashMap;
use indexmap::IndexSet;
use serde::{Deserialize, de::IntoDeserializer};
use toml::{Spanned, de::DeTable};
use yazi_codegen::DeserializeOver;

use super::OpenerRule;

#[derive(Debug, Deserialize, DeserializeOver)]
pub struct Opener(HashMap<String, Vec<OpenerRule>>);

impl Deref for Opener {
	type Target = HashMap<String, Vec<OpenerRule>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Opener {
	pub fn all<'a, I>(&self, uses: I) -> impl Iterator<Item = &OpenerRule>
	where
		I: Iterator<Item = &'a str>,
	{
		uses.flat_map(|use_| self.get(use_)).flatten()
	}

	pub fn first<'a, I>(&self, uses: I) -> Option<&OpenerRule>
	where
		I: Iterator<Item = &'a str>,
	{
		uses.flat_map(|use_| self.get(use_)).flatten().next()
	}

	pub fn block<'a, I>(&self, uses: I) -> Option<&OpenerRule>
	where
		I: Iterator<Item = &'a str>,
	{
		uses.flat_map(|use_| self.get(use_)).flatten().find(|&o| o.block)
	}
}

impl Opener {
	pub(crate) fn reshape(mut self) -> Result<Self> {
		for rules in self.0.values_mut() {
			*rules = mem::take(rules)
				.into_iter()
				.filter(|r| r.r#for.matches())
				.map(|r| r.reshape())
				.collect::<Result<IndexSet<_>>>()?
				.into_iter()
				.collect();
		}

		Ok(self)
	}

	pub(crate) fn deserialize_over_with<'de>(
		mut self,
		table: Spanned<DeTable<'de>>,
	) -> Result<Self, toml::de::Error> {
		for (key, value) in table.into_inner() {
			self.0.insert(key.into_inner().into_owned(), <_>::deserialize(value.into_deserializer())?);
		}

		Ok(self)
	}
}
