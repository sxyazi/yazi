use std::{borrow::Cow, fmt, fmt::Debug};

use anyhow::bail;
use dyn_clone::DynClone;
use hashbrown::HashMap;
use mlua::{Lua, Table};
use serde::Deserialize;
use strum::{EnumString, IntoStaticStr};
use yazi_binding::Scope;
use yazi_macro::impl_data_any;
use yazi_runner::{PluginJob, loader::Chunk};
use yazi_scheduler::plugin::PluginInEntry;
use yazi_shared::{data::{Data, DataKey}, event::{ActionCow, Cmd, FromAction}, id::Id};
use yazi_shim::SStr;

use crate::Ctx;

#[derive(Clone, Debug, Default)]
pub struct PluginOpt {
	pub tab:      Id,
	pub name:     SStr,
	pub args:     HashMap<DataKey, Data>,
	pub mode:     PluginMode,
	pub method:   PluginMethod,
	pub scope:    Scope,
	pub callback: Option<Box<dyn PluginCallback>>,
}

impl_data_any!(PluginOpt);

impl FromAction<Ctx<'_>> for PluginOpt {
	fn from_action(mut a: ActionCow, cx: &Ctx) -> anyhow::Result<Self> {
		let Some(name) = a.take_first::<SStr>().ok().filter(|s| !s.is_empty()) else {
			bail!("plugin name cannot be empty");
		};

		let args = if let Ok(s) = a.second() {
			let (words, last) = yazi_shared::shell::unix::split(s, true)?;
			Cmd::parse_args(words, last)?
		} else {
			a.take_second().unwrap_or_default()
		};

		Ok(Self {
			tab: cx.tab().id,
			name: Self::normalize_name(name),
			args,
			mode: a.str("mode").parse().unwrap_or_default(),
			method: a.str("method").parse().unwrap_or_default(),
			scope: a.take_any("scope").unwrap_or_default(),
			callback: a.take_any("callback"),
		})
	}
}

impl From<PluginOpt> for PluginInEntry {
	fn from(value: PluginOpt) -> Self {
		Self { tab: value.tab, plugin: value.name, args: value.args, ..Default::default() }
	}
}

impl PluginOpt {
	pub fn new_callback<J, F>(job: J, f: F) -> Self
	where
		J: PluginJob + Clone + Send + Sync + 'static,
		F: FnOnce(&Lua, Table, J) -> mlua::Result<()> + Clone + Send + Sync + 'static,
	{
		Self {
			tab: job.tab(),
			name: Self::normalize_name(job.name().clone()),
			mode: PluginMode::Sync,
			callback: Some(Box::new(move |lua, plugin| f(lua, plugin, job))),
			..Default::default()
		}
	}

	pub fn effective_mode(&self, chunk: &Chunk) -> PluginMode {
		self.mode.auto_then(match self.method {
			PluginMethod::Entry => chunk.sync_entry,
			PluginMethod::Peek => chunk.sync_peek,
			PluginMethod::Seek => true,
		})
	}

	fn normalize_name(s: SStr) -> SStr {
		match s {
			Cow::Borrowed(s) => s.strip_suffix(".main").unwrap_or(s).into(),
			Cow::Owned(mut s) => {
				s.truncate(s.strip_suffix(".main").unwrap_or(&s).len());
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
	fn auto_then(self, sync: bool) -> Self {
		if self != Self::Auto {
			return self;
		}
		if sync { Self::Sync } else { Self::Async }
	}
}

// --- Method
#[derive(Clone, Copy, Debug, Default, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum PluginMethod {
	#[default]
	Entry,
	Peek,
	Seek,
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

impl Debug for dyn PluginCallback {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("PluginCallback").finish_non_exhaustive()
	}
}
