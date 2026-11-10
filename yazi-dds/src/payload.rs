use std::{fmt::Display, io::Write, str::FromStr};

use anyhow::{Result, anyhow};
use mlua::{IntoLua, Lua, Value};
use yazi_boot::BOOT;
use yazi_macro::{emit, impl_data_any, relay};
use yazi_shared::{event::{ActionCow, FromAction}, id::Id};

use crate::ember::Ember;

#[derive(Clone, Debug)]
pub struct Payload<'a> {
	pub receiver: Id,
	pub sender:   Id,
	pub body:     Ember<'a>,
}

impl_data_any!(Payload<'static>);

impl<'a> Payload<'a> {
	pub fn new(body: Ember<'a>) -> Self { Self { receiver: Id::ZERO, sender: BOOT.id, body } }

	pub fn tab(&self) -> Id {
		if self.receiver != 0 && self.receiver == self.sender { self.body.tab() } else { Id::ZERO }
	}

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
	pub(super) fn emit(self) {
		emit!(Call(relay!(app:accept_payload).with_any("payload", self)));
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

impl<C> FromAction<C> for Payload<'_> {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		a.take_any2("payload").ok_or_else(|| anyhow!("Missing 'payload' in Payload"))?
	}
}

impl Display for Payload<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let body = serde_json::to_string(&self.body).map_err(|_| std::fmt::Error)?;

		write!(f, "{},{},{},{body}", self.body.kind(), self.receiver, self.sender)
	}
}

impl<'a> IntoLua for Payload<'a> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("receiver", self.receiver.into_lua(lua)?),
				("sender", self.sender.into_lua(lua)?),
				("body", self.body.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
