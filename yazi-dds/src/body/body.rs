use anyhow::{Result, bail};
use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::Serialize;
use yazi_shared::Id;

use super::{BodyBulk, BodyBye, BodyCd, BodyCustom, BodyDelete, BodyHey, BodyHi, BodyHover, BodyLoad, BodyMount, BodyMove, BodyRename, BodyTab, BodyTrash, BodyYank};
use crate::Payload;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Body<'a> {
	Hi(BodyHi<'a>),
	Hey(BodyHey),
	Bye(BodyBye),
	Tab(BodyTab),
	Cd(BodyCd<'a>),
	Load(BodyLoad<'a>),
	Hover(BodyHover<'a>),
	Rename(BodyRename<'a>),
	Bulk(BodyBulk<'a>),
	Yank(BodyYank<'a>),
	Move(BodyMove<'a>),
	Trash(BodyTrash<'a>),
	Delete(BodyDelete<'a>),
	Mount(BodyMount),
	Custom(BodyCustom),
}

impl Body<'static> {
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
			"move" => Self::Move(serde_json::from_str(body)?),
			"trash" => Self::Trash(serde_json::from_str(body)?),
			"delete" => Self::Delete(serde_json::from_str(body)?),
			"mount" => Self::Mount(serde_json::from_str(body)?),
			_ => BodyCustom::from_str(kind, body)?,
		})
	}

	pub fn from_lua(kind: &str, value: Value) -> mlua::Result<Self> {
		Self::validate(kind).into_lua_err()?;
		BodyCustom::from_lua(kind, value)
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
				| "move"
				| "trash"
				| "delete"
				| "mount"
		) {
			bail!("Cannot construct system event");
		}

		let mut it = kind.bytes().peekable();
		if it.peek() == Some(&b'@') {
			it.next(); // Skip `@` as it's a prefix for static messages
		}
		if !it.all(|b| b.is_ascii_alphanumeric() || b == b'-') {
			bail!("Kind must be alphanumeric with dashes");
		}

		Ok(())
	}
}

impl<'a> Body<'a> {
	#[inline]
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
			Self::Move(_) => "move",
			Self::Trash(_) => "trash",
			Self::Delete(_) => "delete",
			Self::Mount(_) => "mount",
			Self::Custom(b) => b.kind.as_str(),
		}
	}

	#[inline]
	pub fn with_receiver(self, receiver: Id) -> Payload<'a> {
		Payload::new(self).with_receiver(receiver)
	}

	#[inline]
	pub fn with_sender(self, sender: Id) -> Payload<'a> { Payload::new(self).with_sender(sender) }
}

impl IntoLua for Body<'static> {
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
			Self::Move(b) => b.into_lua(lua),
			Self::Trash(b) => b.into_lua(lua),
			Self::Delete(b) => b.into_lua(lua),
			Self::Mount(b) => b.into_lua(lua),
			Self::Custom(b) => b.into_lua(lua),
		}
	}
}
