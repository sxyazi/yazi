use std::{collections::HashMap, ops::Deref};

use anyhow::Result;
use serde::{Deserialize, Deserializer};
use yazi_shared::Xdg;

use crate::providers::Provider;

pub struct Providers(HashMap<String, opendal::Operator>);

impl Deref for Providers {
	type Target = HashMap<String, opendal::Operator>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Default for Providers {
	fn default() -> Self {
		let s = std::fs::read_to_string(Xdg::state_dir().join(".secret.toml")).unwrap_or_default();
		toml::from_str(&s).unwrap()
	}
}

impl<'de> Deserialize<'de> for Providers {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize, Default)]
		struct Outer {
			providers: Shadow,
		}
		#[derive(Deserialize, Default)]
		struct Shadow {
			rules: Vec<Provider>,
		}

		let outer = Outer::deserialize(deserializer)?;
		let result: Result<_> =
			outer.providers.rules.into_iter().map(|p| Ok((p.name.clone(), p.try_into()?))).collect();

		match result {
			Ok(v) => Ok(Providers(v)),
			Err(e) => Err(serde::de::Error::custom(e)),
		}
	}
}
