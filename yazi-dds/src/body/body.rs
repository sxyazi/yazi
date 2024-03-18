use anyhow::Result;
use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::{BodyBulk, BodyCd, BodyCustom, BodyHey, BodyHi, BodyHover, BodyRename, BodyTabs, BodyYank};
use crate::Payload;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Body<'a> {
	Hi(BodyHi),
	Hey(BodyHey),
	Tabs(BodyTabs<'a>),
	Cd(BodyCd<'a>),
	Hover(BodyHover<'a>),
	Rename(BodyRename<'a>),
	Bulk(BodyBulk<'a>),
	Yank(BodyYank<'a>),
	Custom(BodyCustom),
}

impl<'a> Body<'a> {
	pub fn from_str(kind: &str, body: &str) -> Result<Self> {
		Ok(match kind {
			"hi" => Body::Hi(serde_json::from_str(body)?),
			"hey" => Body::Hey(serde_json::from_str(body)?),
			"tabs" => Body::Tabs(serde_json::from_str(body)?),
			"cd" => Body::Cd(serde_json::from_str(body)?),
			"hover" => Body::Hover(serde_json::from_str(body)?),
			"rename" => Body::Rename(serde_json::from_str(body)?),
			"bulk" => Body::Bulk(serde_json::from_str(body)?),
			"yank" => Body::Yank(serde_json::from_str(body)?),
			_ => BodyCustom::from_str(kind, body)?,
		})
	}

	pub fn from_lua(kind: &str, value: Value) -> Result<Self> {
		Ok(match kind {
			"hi" | "hey" | "tabs" | "cd" | "hover" | "rename" | "bulk" | "yank" => {
				Err("Cannot construct system event from Lua").into_lua_err()?
			}
			_ => BodyCustom::from_lua(kind, value)?,
		})
	}

	#[inline]
	pub fn kind(&self) -> &str {
		match self {
			Self::Hi(_) => "hi",
			Self::Hey(_) => "hey",
			Self::Tabs(_) => "tabs",
			Self::Cd(_) => "cd",
			Self::Hover(_) => "hover",
			Self::Rename(_) => "rename",
			Self::Bulk(_) => "bulk",
			Self::Yank(_) => "yank",
			Self::Custom(b) => b.kind.as_str(),
		}
	}

	pub fn tab(kind: &str, body: &str) -> usize {
		match kind {
			"cd" | "hover" | "bulk" | "rename" => {}
			_ => return 0,
		}

		match Self::from_str(kind, body) {
			Ok(Body::Cd(b)) => b.tab,
			Ok(Body::Hover(b)) => b.tab,
			Ok(Body::Bulk(b)) => b.tab,
			Ok(Body::Rename(b)) => b.tab,
			_ => 0,
		}
	}

	pub fn upgrade(self) -> Payload<'a> {
		let severity = match self {
			Body::Hi(_) => 0,
			Body::Hey(_) => 0,
			Body::Tabs(_) => 10,
			Body::Cd(_) => 20,
			Body::Hover(_) => 30,
			Body::Rename(_) => 0,
			Body::Bulk(_) => 0,
			Body::Yank(_) => 40,
			Body::Custom(_) => 0,
		};
		Payload::new(self).with_severity(severity)
	}
}

impl IntoLua<'_> for Body<'static> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			Body::Hi(b) => b.into_lua(lua),
			Body::Hey(b) => b.into_lua(lua),
			Body::Tabs(b) => b.into_lua(lua),
			Body::Cd(b) => b.into_lua(lua),
			Body::Hover(b) => b.into_lua(lua),
			Body::Rename(b) => b.into_lua(lua),
			Body::Bulk(b) => b.into_lua(lua),
			Body::Yank(b) => b.into_lua(lua),
			Body::Custom(b) => b.into_lua(lua),
		}
	}
}
