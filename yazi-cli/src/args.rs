use std::borrow::Cow;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand, command};
use yazi_shared::event::Cmd;

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
	/// Emit a command to be executed by the current instance.
	Emit(CommandEmit),
	/// Emit a command to be executed by the specified instance.
	EmitTo(CommandEmitTo),
	/// Manage packages.
	Pack(CommandPack),
	/// Publish a message to the current instance.
	Pub(CommandPub),
	/// Publish a message to the specified instance.
	PubTo(CommandPubTo),
	/// Subscribe to messages from all remote instances.
	Sub(CommandSub),
}

#[derive(clap::Args)]
pub(super) struct CommandEmit {
	/// The name of the command.
	pub(super) name: String,
	/// The arguments of the command.
	#[arg(allow_hyphen_values = true, trailing_var_arg = true)]
	pub(super) args: Vec<String>,
}

#[derive(clap::Args)]
pub(super) struct CommandEmitTo {
	/// The receiver ID.
	pub(super) receiver: u64,
	/// The name of the command.
	pub(super) name:     String,
	/// The arguments of the command.
	#[arg(allow_hyphen_values = true, trailing_var_arg = true)]
	pub(super) args:     Vec<String>,
}

#[derive(clap::Args)]
#[command(arg_required_else_help = true)]
pub(super) struct CommandPack {
	/// Add packages.
	#[arg(short = 'a', long, num_args = 1..)]
	pub(super) add:     Option<Vec<String>>,
	/// Delete packages.
	#[arg(short = 'd', long, num_args = 1..)]
	pub(super) delete:  Option<Vec<String>>,
	/// Install all packages.
	#[arg(short = 'i', long)]
	pub(super) install: bool,
	/// List all packages.
	#[arg(short = 'l', long)]
	pub(super) list:    bool,
	/// Upgrade all packages.
	#[arg(short = 'u', long)]
	pub(super) upgrade: bool,
	/// Migrate all packages.
	#[arg(short = 'm', long)]
	pub(super) migrate: bool, // TODO: remove this
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
	/// Send the message as a list of strings.
	#[arg(long, num_args = 0..)]
	pub(super) list: Vec<String>,
}

impl CommandPub {
	#[allow(dead_code)]
	pub(super) fn receiver() -> Result<u64> {
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
	/// Send the message as a list of strings.
	#[arg(long, num_args = 0..)]
	pub(super) list:     Vec<String>,
}

#[derive(clap::Args)]
pub(super) struct CommandSub {
	/// The kind of messages to subscribe to, separated by commas if multiple.
	#[arg(index = 1)]
	pub(super) kinds: String,
}

// --- Macros
macro_rules! impl_emit_body {
	($name:ident) => {
		impl $name {
			#[allow(dead_code)]
			pub(super) fn body(self) -> Result<String> {
				Ok(serde_json::to_string(&(self.name, Cmd::parse_args(self.args.into_iter(), false)?))?)
			}
		}
	};
}

macro_rules! impl_pub_body {
	($name:ident) => {
		impl $name {
			#[allow(dead_code)]
			pub(super) fn body(&self) -> Result<Cow<str>> {
				Ok(if let Some(json) = &self.json {
					json.into()
				} else if let Some(str) = &self.str {
					serde_json::to_string(str)?.into()
				} else if !self.list.is_empty() {
					serde_json::to_string(&self.list)?.into()
				} else {
					"".into()
				})
			}
		}
	};
}

impl_emit_body!(CommandEmit);
impl_emit_body!(CommandEmitTo);

impl_pub_body!(CommandPub);
impl_pub_body!(CommandPubTo);
