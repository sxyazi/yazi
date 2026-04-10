use std::borrow::Cow;

use hashbrown::HashMap;
use mlua::{ExternalError, FromLua, Lua, Value};
use yazi_dds::Sendable;
use yazi_runner::entry::EntryJob;
use yazi_shared::{Id, SStr, data::{Data, DataKey}};

use crate::{TaskIn, plugin::PluginProgEntry};

#[derive(Debug)]
pub(crate) enum PluginIn {
	Entry(PluginInEntry),
}

impl TaskIn for PluginIn {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Entry(r#in) => r#in.id(),
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Entry(r#in) => _ = r#in.set_id(id),
		}
		self
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Entry(r#in) => r#in.title(),
		}
	}
}

impl_from_in!(Entry(PluginInEntry));

// --- Entry
#[derive(Clone, Debug, Default)]
pub struct PluginInEntry {
	pub id:     Id,
	pub plugin: SStr,
	pub args:   HashMap<DataKey, Data>,
	pub title:  SStr,
	pub track:  bool,
}

impl TaskIn for PluginInEntry {
	type Prog = PluginProgEntry;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		if self.title.is_empty() {
			format!("Run plugin '{}'", self.plugin).into()
		} else {
			Cow::Borrowed(&self.title)
		}
	}

	fn set_title(&mut self, title: impl Into<SStr>) -> &mut Self {
		self.title = title.into();
		self
	}
}

impl PluginInEntry {
	pub(crate) fn into_job(self) -> EntryJob {
		EntryJob { id: self.id, args: self.args, plugin: self.plugin }
	}
}

impl FromLua for PluginInEntry {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let Value::Table(t) = value else {
			return Err("constructing PluginInEntry from non-table value".into_lua_err());
		};

		Ok(Self {
			plugin: t.raw_get::<String>(1)?.into(),
			args: Sendable::table_to_args(lua, t.raw_get("args")?)?,
			track: t.raw_get("track")?,
			..Default::default()
		})
	}
}
