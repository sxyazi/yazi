use std::{collections::HashMap, mem, ops::Deref};

use anyhow::Result;
use indexmap::IndexSet;
use serde::Deserialize;

use super::OpenerRule;
use crate::check_for;

#[derive(Debug, Deserialize)]
pub struct Opener(HashMap<String, Vec<OpenerRule>>);

impl Deref for Opener {
	type Target = HashMap<String, Vec<OpenerRule>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Opener {
	pub fn all<'a, I>(&'a self, uses: I) -> Vec<&'a OpenerRule>
	where
		I: Iterator<Item = &'a str>,
	{
		uses.flat_map(|use_| self.get(use_)).flatten().collect()
	}

	pub fn first<'a, 'b, I>(&'a self, uses: I) -> Option<&'a OpenerRule>
	where
		I: Iterator<Item = &'b str>,
	{
		uses.flat_map(|use_| self.get(use_)).flatten().next()
	}

	pub fn block<'a, I>(&'a self, uses: I) -> Option<&'a OpenerRule>
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
				.map(|mut r| (r.r#for.take(), r))
				.filter(|(r#for, _)| check_for(r#for.as_deref()))
				.map(|(_, r)| r.reshape())
				.collect::<Result<IndexSet<_>>>()?
				.into_iter()
				.collect();
		}

		Ok(self)
	}

	pub(crate) fn deserialize_over<'de, D>(mut self, deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let map: HashMap<String, Vec<OpenerRule>> = HashMap::deserialize(deserializer)?;
		self.0.extend(map);

		Ok(self)
	}
}
