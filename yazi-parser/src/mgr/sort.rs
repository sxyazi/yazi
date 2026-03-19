use std::str::FromStr;

use anyhow::bail;
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::SER_OPT;
use yazi_fs::{SortBy, SortFallback};
use yazi_shared::{data::Data, event::ActionCow};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SortOpt {
	pub by:        Option<SortBy>,
	pub reverse:   Option<SortBoolState>,
	pub dir_first: Option<SortBoolState>,
	pub sensitive: Option<bool>,
	pub translit:  Option<SortBoolState>,
	pub fallback:  Option<SortFallback>,
}

impl TryFrom<ActionCow> for SortOpt {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self {
			by:        a.first().ok().map(str::parse).transpose()?,
			reverse:   a.get("reverse").ok(),
			dir_first: a.get("dir-first").ok(),
			sensitive: a.get("sensitive").ok(),
			translit:  a.get("translit").ok(),
			fallback:  a.get("fallback").ok().map(str::parse).transpose()?,
		})
	}
}

impl FromLua for SortOpt {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for SortOpt {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}

// --- State
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SortBoolState {
	#[default]
	None,
	On,
	Off,
	Toggle,
}

impl serde::Serialize for SortBoolState {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::On => serializer.serialize_bool(true),
			Self::Off => serializer.serialize_bool(false),
			Self::Toggle => serializer.serialize_str("toggle"),
			Self::None => serializer.serialize_str("none"),
		}
	}
}

impl FromStr for SortBoolState {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"yes" => Ok(Self::On),
			"no" => Ok(Self::Off),
			"toggle" => Ok(Self::Toggle),
			_ => bail!("unknown sort bool state: {s:?}"),
		}
	}
}

impl<'de> serde::Deserialize<'de> for SortBoolState {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		use serde::de::{self, Visitor};

		struct V;

		impl<'de> Visitor<'de> for V {
			type Value = SortBoolState;

			fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				f.write_str(r#"a boolean or one of "yes", "no", "toggle""#)
			}

			fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
				Ok(if v { SortBoolState::On } else { SortBoolState::Off })
			}

			fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
				v.parse().map_err(de::Error::custom)
			}

			fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
				self.visit_str(&v)
			}
		}

		deserializer.deserialize_any(V)
	}
}

impl SortBoolState {
	pub fn bool(self, old: bool) -> bool {
		match self {
			Self::None => old,
			Self::On => true,
			Self::Off => false,
			Self::Toggle => !old,
		}
	}
}

impl TryFrom<&Data> for SortBoolState {
	type Error = anyhow::Error;

	fn try_from(value: &Data) -> Result<Self, Self::Error> {
		match value {
			Data::Boolean(true) => Ok(Self::On),
			Data::Boolean(false) => Ok(Self::Off),
			Data::String(s) => s.parse(),
			_ => bail!("not a valid bool state"),
		}
	}
}
