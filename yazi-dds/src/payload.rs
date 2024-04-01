use std::{fmt::Display, io::Write, str::FromStr};

use anyhow::{anyhow, Result};
use yazi_boot::BOOT;
use yazi_shared::{emit, event::Cmd, Layer};

use crate::{body::Body, ID};

#[derive(Debug)]
pub struct Payload<'a> {
	pub receiver: u64,
	pub severity: u8,
	pub sender:   u64,
	pub body:     Body<'a>,
}

impl<'a> Payload<'a> {
	pub(super) fn new(body: Body<'a>) -> Self { Self { receiver: 0, severity: 0, sender: *ID, body } }

	pub(super) fn flush(&self) { writeln!(std::io::stdout(), "{self}").ok(); }

	pub(super) fn try_flush(&self) {
		let b = if self.receiver == 0 {
			BOOT.remote_events.contains(self.body.kind())
		} else if let Body::Custom(b) = &self.body {
			BOOT.local_events.contains(&b.kind)
		} else {
			false
		};

		if b {
			self.flush();
		}
	}

	pub(super) fn with_receiver(mut self, receiver: u64) -> Self {
		self.receiver = receiver;
		self
	}

	pub(super) fn with_severity(mut self, severity: u8) -> Self {
		self.severity = severity;
		self
	}
}

impl Payload<'static> {
	pub(super) fn emit(self) {
		self.try_flush();
		emit!(Call(Cmd::new("accept_payload").with_data(self), Layer::App));
	}
}

impl FromStr for Payload<'_> {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut parts = s.splitn(5, ',');

		let kind = parts.next().ok_or_else(|| anyhow!("empty kind"))?;

		let receiver =
			parts.next().and_then(|s| s.parse().ok()).ok_or_else(|| anyhow!("invalid receiver"))?;

		let severity =
			parts.next().and_then(|s| s.parse().ok()).ok_or_else(|| anyhow!("invalid severity"))?;

		let sender =
			parts.next().and_then(|s| s.parse().ok()).ok_or_else(|| anyhow!("invalid sender"))?;

		let body = parts.next().ok_or_else(|| anyhow!("empty body"))?;

		Ok(Self { receiver, severity, sender, body: Body::from_str(kind, body)? })
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
			Body::Cd(b) => serde_json::to_string(b),
			Body::Hover(b) => serde_json::to_string(b),
			Body::Rename(b) => serde_json::to_string(b),
			Body::Bulk(b) => serde_json::to_string(b),
			Body::Yank(b) => serde_json::to_string(b),
			Body::Custom(b) => serde_json::to_string(b),
		};

		if let Ok(s) = result {
			write!(f, "{},{},{},{},{s}", self.body.kind(), self.receiver, self.severity, self.sender)
		} else {
			Err(std::fmt::Error)
		}
	}
}
