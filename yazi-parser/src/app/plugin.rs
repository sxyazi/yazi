use std::{borrow::Cow, fmt::Debug, str::FromStr};

use anyhow::bail;
use hashbrown::HashMap;
use mlua::{Lua, Table};
use serde::Deserialize;
use yazi_shared::{SStr, data::{Data, DataKey}, event::{Cmd, CmdCow}};

pub type PluginCallback = Box<dyn FnOnce(&Lua, Table) -> mlua::Result<()> + Send + Sync>;

#[derive(Default)]
pub struct PluginOpt {
	pub id:   SStr,
	pub args: HashMap<DataKey, Data>,
	pub mode: PluginMode,
	pub cb:   Option<PluginCallback>,
}

impl TryFrom<CmdCow> for PluginOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any("opt") {
			return Ok(opt);
		}

		let Some(id) = c.take_first::<SStr>().ok().filter(|s| !s.is_empty()) else {
			bail!("plugin id cannot be empty");
		};

		let args = if let Ok(s) = c.second() {
			let (words, last) = yazi_shared::shell::unix::split(s, true)?;
			Cmd::parse_args(words, last)?
		} else {
			Default::default()
		};

		let mode = c.str("mode").parse().unwrap_or_default();
		Ok(Self { id: Self::normalize_id(id), args, mode, cb: c.take_any("callback") })
	}
}

impl Debug for PluginOpt {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("PluginOpt")
			.field("id", &self.id)
			.field("args", &self.args)
			.field("mode", &self.mode)
			.field("cb", &self.cb.is_some())
			.finish()
	}
}

impl PluginOpt {
	pub fn new_callback(id: impl Into<SStr>, cb: PluginCallback) -> Self {
		Self {
			id: Self::normalize_id(id.into()),
			mode: PluginMode::Sync,
			cb: Some(cb),
			..Default::default()
		}
	}

	fn normalize_id(s: SStr) -> SStr {
		match s {
			Cow::Borrowed(s) => s.trim_end_matches(".main").into(),
			Cow::Owned(mut s) => {
				s.truncate(s.trim_end_matches(".main").len());
				s.into()
			}
		}
	}
}

// --- Mode
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginMode {
	#[default]
	Auto,
	Sync,
	Async,
}

impl FromStr for PluginMode {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}

impl PluginMode {
	pub fn auto_then(self, sync: bool) -> Self {
		if self != Self::Auto {
			return self;
		}
		if sync { Self::Sync } else { Self::Async }
	}
}
