use std::{fmt::Display, io::Write, str::FromStr};

use anyhow::{Result, anyhow};
use mlua::{IntoLua, Lua, Value};
use yazi_boot::BOOT;
use yazi_macro::{emit, relay};
use yazi_shared::{Id, event::CmdCow};

use crate::{ID, ember::Ember, spark::Spark};

#[derive(Clone, Debug)]
pub struct Payload<'a> {
	pub receiver: Id,
	pub sender:   Id,
	pub body:     Ember<'a>,
}

impl<'a> Payload<'a> {
	pub(super) fn new(body: Ember<'a>) -> Self { Self { receiver: Id(0), sender: *ID, body } }

	pub(super) fn flush(&self) -> Result<()> {
		writeln!(std::io::stdout(), "{self}")?;
		Ok(())
	}

	pub(super) fn try_flush(&self) -> Result<()> {
		let b = if self.receiver == 0 {
			BOOT.remote_events.contains(self.body.kind())
		} else if let Ember::Custom(b) = &self.body {
			BOOT.local_events.contains(&b.kind)
		} else {
			false
		};
		if b { self.flush() } else { Ok(()) }
	}

	pub(super) fn with_receiver(mut self, receiver: Id) -> Self {
		self.receiver = receiver;
		self
	}

	pub(super) fn with_sender(mut self, sender: Id) -> Self {
		self.sender = sender;
		self
	}
}

impl Payload<'static> {
	pub(super) fn emit(self) -> Result<()> {
		self.try_flush()?;
		emit!(Call(relay!(app:accept_payload).with_any("payload", self)));
		Ok(())
	}
}

impl FromStr for Payload<'static> {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut parts = s.splitn(4, ',');

		let kind = parts.next().ok_or_else(|| anyhow!("empty kind"))?;

		let receiver =
			parts.next().and_then(|s| s.parse().ok()).ok_or_else(|| anyhow!("invalid receiver"))?;

		let sender =
			parts.next().and_then(|s| s.parse().ok()).ok_or_else(|| anyhow!("invalid sender"))?;

		let body = parts.next().ok_or_else(|| anyhow!("empty body"))?;

		Ok(Self { receiver, sender, body: Ember::from_str(kind, body)? })
	}
}

impl<'a> From<Ember<'a>> for Payload<'a> {
	fn from(value: Ember<'a>) -> Self { Self::new(value) }
}

impl<'a> TryFrom<Spark<'a>> for Payload<'a> {
	type Error = ();

	fn try_from(value: Spark<'a>) -> Result<Self, Self::Error> {
		match value {
			Spark::AppAcceptPayload(payload) => Ok(payload),
			_ => Err(()),
		}
	}
}

impl TryFrom<CmdCow> for Payload<'_> {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		c.take_any2("payload").ok_or_else(|| anyhow!("Missing 'payload' in Payload"))?
	}
}

impl Display for Payload<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let result = match &self.body {
			Ember::Hi(b) => serde_json::to_string(b),
			Ember::Hey(b) => serde_json::to_string(b),
			Ember::Bye(b) => serde_json::to_string(b),
			Ember::Cd(b) => serde_json::to_string(b),
			Ember::Load(b) => serde_json::to_string(b),
			Ember::Hover(b) => serde_json::to_string(b),
			Ember::Tab(b) => serde_json::to_string(b),
			Ember::Rename(b) => serde_json::to_string(b),
			Ember::Bulk(b) => serde_json::to_string(b),
			Ember::Yank(b) => serde_json::to_string(b),
			Ember::Duplicate(b) => serde_json::to_string(b),
			Ember::Move(b) => serde_json::to_string(b),
			Ember::Trash(b) => serde_json::to_string(b),
			Ember::Delete(b) => serde_json::to_string(b),
			Ember::Download(b) => serde_json::to_string(b),
			Ember::Mount(b) => serde_json::to_string(b),
			Ember::Custom(b) => serde_json::to_string(b),
		};

		if let Ok(s) = result {
			write!(f, "{},{},{},{s}", self.body.kind(), self.receiver, self.sender)
		} else {
			Err(std::fmt::Error)
		}
	}
}

impl<'a> IntoLua for Payload<'a> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("receiver", yazi_binding::Id(self.receiver).into_lua(lua)?),
				("sender", yazi_binding::Id(self.sender).into_lua(lua)?),
				("body", self.body.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
