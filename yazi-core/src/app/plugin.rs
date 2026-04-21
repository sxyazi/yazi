use std::{borrow::Cow, fmt, fmt::Debug};

use anyhow::bail;
use dyn_clone::DynClone;
use hashbrown::HashMap;
use mlua::{Lua, Table};
use serde::Deserialize;
use strum::EnumString;
use yazi_scheduler::plugin::PluginInEntry;
use yazi_shared::{SStr, data::{Data, DataKey}, event::{ActionCow, Cmd}};

#[derive(Clone, Debug, Default)]
pub struct PluginOpt {
	pub id:       SStr,
	pub args:     HashMap<DataKey, Data>,
	pub mode:     PluginMode,
	pub callback: Option<Box<dyn PluginCallback>>,
}

impl TryFrom<ActionCow> for PluginOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Some(id) = a.take_first::<SStr>().ok().filter(|s| !s.is_empty()) else {
			bail!("plugin id cannot be empty");
		};

		let args = if let Ok(s) = a.second() {
			let (words, last) = yazi_shared::shell::unix::split(s, true)?;
			Cmd::parse_args(words, last)?
		} else {
			Default::default()
		};

		let mode = a.str("mode").parse().unwrap_or_default();
		Ok(Self { id: Self::normalize_id(id), args, mode, callback: a.take_any("callback") })
	}
}

impl From<PluginOpt> for PluginInEntry {
	fn from(value: PluginOpt) -> Self {
		Self { plugin: value.id, args: value.args, ..Default::default() }
	}
}

impl PluginOpt {
	pub fn new_callback(id: impl Into<SStr>, f: impl PluginCallback) -> Self {
		Self {
			id: Self::normalize_id(id.into()),
			mode: PluginMode::Sync,
			callback: Some(Box::new(f)),
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
#[derive(Clone, Copy, Debug, Default, Deserialize, EnumString, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum PluginMode {
	#[default]
	Auto,
	Sync,
	Async,
}

impl PluginMode {
	pub fn auto_then(self, sync: bool) -> Self {
		if self != Self::Auto {
			return self;
		}
		if sync { Self::Sync } else { Self::Async }
	}
}

// --- Callback
pub trait PluginCallback:
	FnOnce(&Lua, Table) -> mlua::Result<()> + Send + Sync + DynClone + 'static
{
}

impl<T> PluginCallback for T where
	T: FnOnce(&Lua, Table) -> mlua::Result<()> + Send + Sync + DynClone + 'static
{
}

impl Clone for Box<dyn PluginCallback> {
	fn clone(&self) -> Self { dyn_clone::clone_box(&**self) }
}

impl fmt::Debug for dyn PluginCallback {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("PluginCallback").finish_non_exhaustive()
	}
}
