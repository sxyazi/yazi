use std::{any::Any, collections::HashMap};

use serde::{Deserialize, Serialize};

use crate::{fs::Url, OrderedFloat};

// --- Arg
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Data {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(f64),
	String(String),
	Table(HashMap<DataKey, Data>),
	#[serde(skip)]
	Url(Url),
	#[serde(skip)]
	Any(Box<dyn Any + Send>),
}

impl Data {
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
			if let (DataKey::String(k), Self::String(v)) = pair {
				map.insert(k, v);
			}
		}
		map
	}
}

// --- Key
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataKey {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(OrderedFloat),
	String(String),
}

impl DataKey {
	#[inline]
	pub fn is_numeric(&self) -> bool { matches!(self, Self::Integer(_) | Self::Number(_)) }
}
