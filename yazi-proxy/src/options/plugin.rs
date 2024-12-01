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

	// TODO: remove this
	pub _old_args: Vec<Data>,
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

		let (args, _old_args) = if let Some(s) = c.str("args") {
			(
				Cmd::parse_args(shell_words::split(s)?.into_iter(), true)?,
				shell_words::split(s)?.into_iter().map(Data::String).collect(),
			)
		} else {
			(Default::default(), Default::default())
		};

		let mut mode = c.str("mode").map(Into::into).unwrap_or_default();
		if c.bool("sync") {
			mode = PluginMode::Sync;
			let s = "The `--sync` option for the `plugin` command has been deprecated in Yazi v0.4.

Please add `--- @sync entry` metadata at the head of your `{id}` plugin instead. See #1891 for details: https://github.com/sxyazi/yazi/pull/1891";
			crate::AppProxy::notify(crate::options::NotifyOpt {
				title:   "Deprecated API".to_owned(),
				content: s.replace("{id}", &id),
				level:   crate::options::NotifyLevel::Warn,
				timeout: std::time::Duration::from_secs(20),
			});
		}

		Ok(Self { id, args, mode, cb: c.take_any("callback"), _old_args })
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
	pub fn new_callback(id: &str, cb: PluginCallback) -> Self {
		Self { id: id.to_owned().into(), mode: PluginMode::Sync, cb: Some(cb), ..Default::default() }
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
