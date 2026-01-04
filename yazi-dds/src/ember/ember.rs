use anyhow::{Result, bail};
use mlua::{ExternalResult, IntoLua, Lua, Value};
use yazi_shared::Id;

use super::{EmberBulk, EmberBye, EmberCd, EmberCustom, EmberDelete, EmberDuplicate, EmberHey, EmberHi, EmberHover, EmberLoad, EmberMount, EmberMove, EmberRename, EmberTab, EmberTrash, EmberYank};
use crate::Payload;

#[derive(Debug)]
pub enum Ember<'a> {
	Hi(EmberHi<'a>),
	Hey(EmberHey),
	Bye(EmberBye),
	Tab(EmberTab),
	Cd(EmberCd<'a>),
	Load(EmberLoad<'a>),
	Hover(EmberHover<'a>),
	Rename(EmberRename<'a>),
	Bulk(EmberBulk<'a>),
	Yank(EmberYank<'a>),
	Duplicate(EmberDuplicate<'a>),
	Move(EmberMove<'a>),
	Trash(EmberTrash<'a>),
	Delete(EmberDelete<'a>),
	Mount(EmberMount),
	Custom(EmberCustom),
}

impl Ember<'static> {
	pub fn from_str(kind: &str, body: &str) -> Result<Self> {
		Ok(match kind {
			"hi" => Self::Hi(serde_json::from_str(body)?),
			"hey" => Self::Hey(serde_json::from_str(body)?),
			"bye" => Self::Bye(serde_json::from_str(body)?),
			"tab" => Self::Tab(serde_json::from_str(body)?),
			"cd" => Self::Cd(serde_json::from_str(body)?),
			"load" => Self::Load(serde_json::from_str(body)?),
			"hover" => Self::Hover(serde_json::from_str(body)?),
			"rename" => Self::Rename(serde_json::from_str(body)?),
			"bulk" => Self::Bulk(serde_json::from_str(body)?),
			"@yank" => Self::Yank(serde_json::from_str(body)?),
			"duplicate" => Self::Duplicate(serde_json::from_str(body)?),
			"move" => Self::Move(serde_json::from_str(body)?),
			"trash" => Self::Trash(serde_json::from_str(body)?),
			"delete" => Self::Delete(serde_json::from_str(body)?),
			"mount" => Self::Mount(serde_json::from_str(body)?),
			_ => EmberCustom::from_str(kind, body)?,
		})
	}

	pub fn from_lua(lua: &Lua, kind: &str, value: Value) -> mlua::Result<Self> {
		Self::validate(kind).into_lua_err()?;
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
				| "hover"
				| "rename"
				| "bulk"
				| "@yank"
				| "duplicate"
				| "move"
				| "trash"
				| "delete"
				| "mount"
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
			Self::Hi(_) => "hi",
			Self::Hey(_) => "hey",
			Self::Bye(_) => "bye",
			Self::Tab(_) => "tab",
			Self::Cd(_) => "cd",
			Self::Load(_) => "load",
			Self::Hover(_) => "hover",
			Self::Rename(_) => "rename",
			Self::Bulk(_) => "bulk",
			Self::Yank(_) => "@yank",
			Self::Duplicate(_) => "duplicate",
			Self::Move(_) => "move",
			Self::Trash(_) => "trash",
			Self::Delete(_) => "delete",
			Self::Mount(_) => "mount",
			Self::Custom(b) => b.kind.as_str(),
		}
	}

	pub fn with_receiver(self, receiver: Id) -> Payload<'a> {
		Payload::new(self).with_receiver(receiver)
	}

	pub fn with_sender(self, sender: Id) -> Payload<'a> { Payload::new(self).with_sender(sender) }
}

impl<'a> IntoLua for Ember<'a> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			Self::Hi(b) => b.into_lua(lua),
			Self::Hey(b) => b.into_lua(lua),
			Self::Bye(b) => b.into_lua(lua),
			Self::Cd(b) => b.into_lua(lua),
			Self::Load(b) => b.into_lua(lua),
			Self::Hover(b) => b.into_lua(lua),
			Self::Tab(b) => b.into_lua(lua),
			Self::Rename(b) => b.into_lua(lua),
			Self::Bulk(b) => b.into_lua(lua),
			Self::Yank(b) => b.into_lua(lua),
			Self::Duplicate(b) => b.into_lua(lua),
			Self::Move(b) => b.into_lua(lua),
			Self::Trash(b) => b.into_lua(lua),
			Self::Delete(b) => b.into_lua(lua),
			Self::Mount(b) => b.into_lua(lua),
			Self::Custom(b) => b.into_lua(lua),
		}
	}
}
