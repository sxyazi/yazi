use std::{fmt::Display, io::Write, str::FromStr};

use anyhow::{Result, anyhow};
use yazi_boot::BOOT;
use yazi_macro::emit;
use yazi_shared::{Id, event::Cmd};

use crate::{ID, body::Body};

#[derive(Debug)]
pub struct Payload<'a> {
	pub receiver: Id,
	pub sender:   Id,
	pub body:     Body<'a>,
}

impl<'a> Payload<'a> {
	pub(super) fn new(body: Body<'a>) -> Self { Self { receiver: Id(0), sender: *ID, body } }

	pub(super) fn flush(&self) -> Result<()> {
		writeln!(std::io::stdout(), "{self}")?;
		Ok(())
	}

	pub(super) fn try_flush(&self) -> Result<()> {
		let b = if self.receiver == 0 {
			BOOT.remote_events.contains(self.body.kind())
		} else if let Body::Custom(b) = &self.body {
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
		emit!(Call(Cmd::new("app:accept_payload").with_any("payload", self)));
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

		Ok(Self { receiver, sender, body: Body::from_str(kind, body)? })
	}
}

impl<'a> From<Body<'a>> for Payload<'a> {
	fn from(value: Body<'a>) -> Self { Self::new(value) }
}

impl Display for Payload<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let result = match &self.body {
			Body::Hi(b) => serde_json::to_string(b),
			Body::Hey(b) => serde_json::to_string(b),
			Body::Bye(b) => serde_json::to_string(b),
			Body::Cd(b) => serde_json::to_string(b),
			Body::Load(b) => serde_json::to_string(b),
			Body::Hover(b) => serde_json::to_string(b),
			Body::Tab(b) => serde_json::to_string(b),
			Body::Rename(b) => serde_json::to_string(b),
			Body::Bulk(b) => serde_json::to_string(b),
			Body::Yank(b) => serde_json::to_string(b),
			Body::Move(b) => serde_json::to_string(b),
			Body::Trash(b) => serde_json::to_string(b),
			Body::Delete(b) => serde_json::to_string(b),
			Body::Mount(b) => serde_json::to_string(b),
			Body::Custom(b) => serde_json::to_string(b),
		};

		if let Ok(s) = result {
			write!(f, "{},{},{},{s}", self.body.kind(), self.receiver, self.sender)
		} else {
			Err(std::fmt::Error)
		}
	}
}
