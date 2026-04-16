use std::ops::Deref;

use hashbrown::HashMap;
use serde::Deserialize;
use toml::{Spanned, de::DeTable};
use yazi_shim::toml::{DeserializeOverHook, DeserializeOverWith, deserialize_spanned};

use super::OpenerRule;

#[derive(Debug, Deserialize)]
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

impl DeserializeOverHook for Opener {
	fn deserialize_over_hook(mut self) -> Result<Self, toml::de::Error> {
		for rules in self.0.values_mut() {
			rules.retain(|r| r.r#for.matches());
			rules.iter_mut().for_each(|r| r.fill());
		}

		Ok(self)
	}
}

impl DeserializeOverWith for Opener {
	fn deserialize_over_with<'de>(
		mut self,
		table: Spanned<DeTable<'de>>,
	) -> Result<Self, toml::de::Error> {
		for (key, value) in table.into_inner() {
			self.0.insert(key.into_inner().into_owned(), deserialize_spanned(value)?);
		}

		Ok(self)
	}
}
