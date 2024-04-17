use std::collections::HashMap;

use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::OrderedFloat;

// --- Arg
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(f64),
	String(String),
	Table(HashMap<ArgKey, Arg>),
}

impl Arg {
	#[inline]
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Self::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	#[inline]
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	pub fn into_table_string(self) -> HashMap<String, String> {
		let Self::Table(table) = self else {
			return Default::default();
		};

		let mut map = HashMap::with_capacity(table.len());
		for pair in table {
			if let (ArgKey::String(k), Self::String(v)) = pair {
				map.insert(k, v);
			}
		}
		map
	}
}

// --- Key
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgKey {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(OrderedFloat),
	String(String),
}

impl TryInto<ArgKey> for Arg {
	type Error = anyhow::Error;

	fn try_into(self) -> Result<ArgKey, Self::Error> {
		Ok(match self {
			Self::Nil => ArgKey::Nil,
			Self::Boolean(v) => ArgKey::Boolean(v),
			Self::Integer(v) => ArgKey::Integer(v),
			Self::Number(v) => ArgKey::Number(OrderedFloat::new(v)),
			Self::String(v) => ArgKey::String(v),
			Self::Table(_) => bail!("table is not supported"),
		})
	}
}
