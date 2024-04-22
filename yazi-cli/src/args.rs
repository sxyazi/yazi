use std::borrow::Cow;

use anyhow::{bail, Result};
use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ya", version, about, long_about = None)]
#[command(propagate_version = true)]
pub(super) struct Args {
	#[command(subcommand)]
	pub(super) command: Command,
}

#[derive(Subcommand)]
pub(super) enum Command {
	/// Publish a message to remote instance(s).
	Pub(CommandPub),
	/// Publish a static message to all remote instances.
	PubStatic(CommandPubStatic),
}

#[derive(clap::Args)]
pub(super) struct CommandPub {
	/// The receiver ID.
	#[arg(index = 1)]
	pub(super) receiver: u64,
	/// The kind of message.
	#[arg(index = 2)]
	pub(super) kind:     String,
	/// Send the message with a string body.
	#[arg(long)]
	pub(super) str:      Option<String>,
	/// Send the message with a JSON body.
	#[arg(long)]
	pub(super) json:     Option<String>,
}

impl CommandPub {
	#[allow(dead_code)]
	pub(super) fn body(&self) -> Result<Cow<str>> {
		if let Some(json) = &self.json {
			Ok(json.into())
		} else if let Some(str) = &self.str {
			Ok(serde_json::to_string(str)?.into())
		} else {
			bail!("No body provided");
		}
	}
}

#[derive(clap::Args)]
pub(super) struct CommandPubStatic {
	/// The severity of the message.
	#[arg(index = 1)]
	pub(super) severity: u16,
	/// The kind of message.
	#[arg(index = 2)]
	pub(super) kind:     String,
	/// Send the message with a string body.
	#[arg(long)]
	pub(super) str:      Option<String>,
	/// Send the message with a JSON body.
	#[arg(long)]
	pub(super) json:     Option<String>,
}

impl CommandPubStatic {
	#[allow(dead_code)]
	pub(super) fn body(&self) -> Result<Cow<str>> {
		if let Some(json) = &self.json {
			Ok(json.into())
		} else if let Some(str) = &self.str {
			Ok(serde_json::to_string(str)?.into())
		} else {
			bail!("No body provided");
		}
	}
}
