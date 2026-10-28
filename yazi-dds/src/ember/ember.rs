use anyhow::{Result, bail};
use mlua::{IntoLua, Lua, Value};
use strum::IntoStaticStr;
use yazi_shared::id::Id;
use yazi_shim::strum::IntoStr;

use super::{EmberBulkRename, EmberBye, EmberCd, EmberCustom, EmberDelete, EmberDownload, EmberDuplicate, EmberHey, EmberHi, EmberHover, EmberInput, EmberLoad, EmberMount, EmberMove, EmberPatch, EmberRename, EmberTab, EmberTheme, EmberTrash, EmberYank};
use crate::Payload;

#[derive(Clone, Debug, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Ember<'a> {
	Hi(EmberHi<'a>),
	Hey(EmberHey),
	Bye(EmberBye),
	Tab(EmberTab),
	Cd(EmberCd<'a>),
	Load(EmberLoad<'a>),
	Patch(EmberPatch<'a>),
	Hover(EmberHover<'a>),
	Rename(EmberRename<'a>),
	BulkRename(EmberBulkRename<'a>),
	#[strum(serialize = "@yank")]
	Yank(EmberYank<'a>),
	Duplicate(EmberDuplicate<'a>),
	Move(EmberMove<'a>),
	Trash(EmberTrash<'a>),
	Delete(EmberDelete<'a>),
	Download(EmberDownload<'a>),
	Input(EmberInput<'a>),
	Mount(EmberMount),
	Theme(EmberTheme),
	Custom(EmberCustom),
}

impl Ember<'static> {
	pub(crate) fn from_str(kind: &str, body: &str) -> Result<Self> {
		Ok(match kind {
			"hi" => Self::Hi(serde_json::from_str(body)?),
			"hey" => Self::Hey(serde_json::from_str(body)?),
			"bye" => Self::Bye(serde_json::from_str(body)?),
			"tab" => Self::Tab(serde_json::from_str(body)?),
			"cd" => Self::Cd(serde_json::from_str(body)?),
			"load" => Self::Load(serde_json::from_str(body)?),
			"patch" => Self::Patch(serde_json::from_str(body)?),
			"hover" => Self::Hover(serde_json::from_str(body)?),
			"rename" => Self::Rename(serde_json::from_str(body)?),
			"bulk-rename" => Self::BulkRename(serde_json::from_str(body)?),
			"@yank" => Self::Yank(serde_json::from_str(body)?),
			"duplicate" => Self::Duplicate(serde_json::from_str(body)?),
			"move" => Self::Move(serde_json::from_str(body)?),
			"trash" => Self::Trash(serde_json::from_str(body)?),
			"delete" => Self::Delete(serde_json::from_str(body)?),
			"download" => Self::Download(serde_json::from_str(body)?),
			"input" => Self::Input(serde_json::from_str(body)?),
			"mount" => Self::Mount(serde_json::from_str(body)?),
			"theme" => Self::Theme(serde_json::from_str(body)?),
			_ => EmberCustom::from_str(kind, body)?,
		})
	}

	pub fn from_lua(lua: &Lua, kind: &str, value: Value) -> mlua::Result<Self> {
		Self::validate(kind)?;
		EmberCustom::from_lua(lua, kind, value)
	}

	pub fn validate(kind: &str) -> Result<()> {
		if matches!(
			kind,
			"hi"
				| "hey"
				| "bye"
				| "tab"
				| "cd"
				| "load"
				| "patch"
				| "hover"
				| "rename"
				| "bulk-rename"
				| "@yank"
				| "duplicate"
				| "move"
				| "trash"
				| "delete"
				| "download"
				| "input"
				| "mount"
				| "theme"
		) || kind.starts_with("key-")
			|| kind.starts_with("ind-")
			|| kind.starts_with("emit-")
			|| kind.starts_with("relay-")
		{
			bail!("Cannot construct system event");
		}

		let mut it = kind.bytes().peekable();
		if it.peek() == Some(&b'@') {
			it.next(); // Skip `@` as it's a prefix for static messages
		}
		if !it.all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-')) {
			bail!("Kind `{kind}` must be in kebab-case");
		}

		Ok(())
	}
}

impl<'a> Ember<'a> {
	pub fn kind(&self) -> &str {
		match self {
			Self::Custom(b) => b.kind.as_str(),
			_ => self.into_str(),
		}
	}

	pub(crate) fn tab(&self) -> Id {
		match self {
			Self::Tab(b) => b.id,
			Self::Cd(b) => b.tab,
			Self::Load(b) => b.tab,
			Self::Patch(b) => b.tab,
			Self::Hover(b) => b.tab,
			Self::Rename(b) => b.tab,
			_ => Id::ZERO,
		}
	}

	pub(crate) fn with_receiver(self, receiver: Id) -> Payload<'a> {
		Payload::new(self).with_receiver(receiver)
	}
}

impl<'a> IntoLua for Ember<'a> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			Self::Hi(b) => b.into_lua(lua),
			Self::Hey(b) => b.into_lua(lua),
			Self::Bye(b) => b.into_lua(lua),
			Self::Cd(b) => b.into_lua(lua),
			Self::Load(b) => b.into_lua(lua),
			Self::Patch(b) => b.into_lua(lua),
			Self::Hover(b) => b.into_lua(lua),
			Self::Tab(b) => b.into_lua(lua),
			Self::Rename(b) => b.into_lua(lua),
			Self::BulkRename(b) => b.into_lua(lua),
			Self::Yank(b) => b.into_lua(lua),
			Self::Duplicate(b) => b.into_lua(lua),
			Self::Move(b) => b.into_lua(lua),
			Self::Trash(b) => b.into_lua(lua),
			Self::Delete(b) => b.into_lua(lua),
			Self::Download(b) => b.into_lua(lua),
			Self::Input(b) => b.into_lua(lua),
			Self::Mount(b) => b.into_lua(lua),
			Self::Theme(b) => b.into_lua(lua),
			Self::Custom(b) => b.into_lua(lua),
		}
	}
}
