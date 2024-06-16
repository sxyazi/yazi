use std::borrow::Cow;

use anyhow::{bail, Result};
use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Ya", about, long_about = None)]
pub(super) struct Args {
	#[command(subcommand)]
	pub(super) command: Command,

	/// Print version
	#[arg(short = 'V', long)]
	pub(super) version: bool,
}

#[derive(Subcommand)]
pub(super) enum Command {
	/// Publish a message to remote instance(s).
	Pub(CommandPub),
	/// Publish a static message to all remote instances.
	PubStatic(CommandPubStatic),
	/// Manage packages.
	Pack(CommandPack),
	/// Subscribe to messages from all remote instances.
	Sub(CommandSub),
}

#[derive(clap::Args)]
pub(super) struct CommandPub {
	/// The kind of message.
	#[arg(index = 1)]
	pub(super) kind:     String,
	/// The receiver ID.
	#[arg(index = 2)]
	pub(super) receiver: Option<u64>,
	/// Send the message with a string body.
	#[arg(long)]
	pub(super) str:      Option<String>,
	/// Send the message with a JSON body.
	#[arg(long)]
	pub(super) json:     Option<String>,
}

impl CommandPub {
	#[allow(dead_code)]
	pub(super) fn receiver(&self) -> Result<u64> {
		if let Some(receiver) = self.receiver {
			Ok(receiver)
		} else if let Ok(s) = std::env::var("YAZI_ID") {
			Ok(s.parse()?)
		} else {
			bail!("No receiver ID provided, also no YAZI_ID environment variable found.")
		}
	}

	#[allow(dead_code)]
	pub(super) fn body(&self) -> Result<Cow<str>> {
		if let Some(json) = &self.json {
			Ok(json.into())
		} else if let Some(str) = &self.str {
			Ok(serde_json::to_string(str)?.into())
		} else {
			Ok("".into())
		}
	}
}

#[derive(clap::Args)]
pub(super) struct CommandPubStatic {
	/// The kind of message.
	#[arg(index = 1)]
	pub(super) kind:     String,
	/// The severity of the message.
	#[arg(index = 2)]
	pub(super) severity: u16,
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
			Ok("".into())
		}
	}
}

#[derive(clap::Args)]
#[command(arg_required_else_help = true)]
pub(super) struct CommandPack {
	/// Add a package.
	#[arg(short = 'a', long)]
	pub(super) add:     Option<String>,
	/// Install all packages.
	#[arg(short = 'i', long)]
	pub(super) install: bool,
	/// List all packages.
	#[arg(short = 'l', long)]
	pub(super) list:    bool,
	/// Upgrade all packages.
	#[arg(short = 'u', long)]
	pub(super) upgrade: bool,
}

#[derive(clap::Args)]
pub(super) struct CommandSub {
	/// The kind of messages to subscribe to, separated by commas if multiple.
	#[arg(index = 1)]
	pub(super) kinds: String,
}
