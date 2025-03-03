use std::{borrow::Cow, collections::HashMap, fmt::Debug};

use anyhow::bail;
use mlua::{Lua, Table};
use yazi_shared::event::{Cmd, CmdCow, Data, DataKey};

pub type PluginCallback = Box<dyn FnOnce(&Lua, Table) -> mlua::Result<()> + Send + Sync>;

#[derive(Default)]
pub struct PluginOpt {
	pub id:   Cow<'static, str>,
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

		let Some(id) = c.take_first_str().filter(|s| !s.is_empty()) else {
			bail!("plugin id cannot be empty");
		};

		let args = if let Some(s) = c.second_str() {
			let (words, last) = yazi_shared::shell::split_unix(s, true)?;
			Cmd::parse_args(words.into_iter(), last, true)?
		} else {
			Default::default()
		};

		let mode = c.str("mode").map(Into::into).unwrap_or_default();
		Ok(Self { id, args, mode, cb: c.take_any("callback") })
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
	pub fn new_callback(id: impl Into<Cow<'static, str>>, cb: PluginCallback) -> Self {
		Self { id: id.into(), mode: PluginMode::Sync, cb: Some(cb), ..Default::default() }
	}
}

// --- Mode
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PluginMode {
	#[default]
	Auto,
	Sync,
	Async,
}

impl From<&str> for PluginMode {
	fn from(s: &str) -> Self {
		match s {
			"sync" => Self::Sync,
			"async" => Self::Async,
			_ => Self::Auto,
		}
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
