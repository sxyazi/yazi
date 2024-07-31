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
	/// Publish a message to the current instance.
	Pub(CommandPub),
	/// Publish a message to the specified instance.
	PubTo(CommandPubTo),
	/// Subscribe to messages from all remote instances.
	Sub(CommandSub),
	/// Manage packages.
	Pack(CommandPack),
}

#[derive(clap::Args)]
pub(super) struct CommandPub {
	/// The kind of message.
	#[arg(index = 1)]
	pub(super) kind: String,
	/// Send the message with a string body.
	#[arg(long)]
	pub(super) str:  Option<String>,
	/// Send the message with a JSON body.
	#[arg(long)]
	pub(super) json: Option<String>,
	/// Send the message as string of list.
	#[arg(long, num_args = 0..)]
	pub(super) list: Vec<String>,
}

impl CommandPub {
	#[allow(dead_code)]
	pub(super) fn receiver(&self) -> Result<u64> {
		if let Some(s) = std::env::var("YAZI_PID").ok().filter(|s| !s.is_empty()) {
			Ok(s.parse()?)
		} else {
			bail!("No `YAZI_ID` environment variable found.")
		}
	}
}

#[derive(clap::Args)]
pub(super) struct CommandPubTo {
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
	/// Send the message as string of list.
	#[arg(long, num_args = 0..)]
	pub(super) list:     Vec<String>,
}

#[derive(clap::Args)]
pub(super) struct CommandSub {
	/// The kind of messages to subscribe to, separated by commas if multiple.
	#[arg(index = 1)]
	pub(super) kinds: String,
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

// --- Macros
macro_rules! impl_body {
	($name:ident) => {
		impl $name {
			#[allow(dead_code)]
			pub(super) fn body(&self) -> Result<Cow<str>> {
				if let Some(json) = &self.json {
					Ok(json.into())
				} else if let Some(str) = &self.str {
					Ok(serde_json::to_string(str)?.into())
				} else if !self.list.is_empty() {
					Ok(serde_json::to_string(&self.list)?.into())
				} else {
					Ok("".into())
				}
			}
		}
	};
}

impl_body!(CommandPub);
impl_body!(CommandPubTo);
