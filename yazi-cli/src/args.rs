use std::{borrow::Cow, ffi::OsString};

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use yazi_shared::{Either, Id};

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
	#[command(subcommand)]
	Pkg(CommandPkg),
	/// Publish a message to the current instance.
	Pub(CommandPub),
	/// Publish a message to the specified instance.
	PubTo(CommandPubTo),
	/// Subscribe to messages from all remote instances.
	Sub(CommandSub),
}

#[derive(clap::Args)]
pub(super) struct CommandEmit {
	/// Name of the command.
	pub(super) name: String,
	/// Arguments of the command.
	#[arg(allow_hyphen_values = true, trailing_var_arg = true)]
	pub(super) args: Vec<OsString>,
}

#[derive(clap::Args)]
pub(super) struct CommandEmitTo {
	/// Receiver ID.
	pub(super) receiver: Id,
	/// Name of the command.
	pub(super) name:     String,
	/// Arguments of the command.
	#[arg(allow_hyphen_values = true, trailing_var_arg = true)]
	pub(super) args:     Vec<OsString>,
}

#[derive(Subcommand)]
pub(super) enum CommandPkg {
	/// Add packages.
	#[command(arg_required_else_help = true)]
	Add {
		/// Packages to add.
		#[arg(index = 1, num_args = 1..)]
		ids: Vec<String>,
	},
	/// Delete packages.
	#[command(arg_required_else_help = true)]
	Delete {
		/// Packages to delete.
		#[arg(index = 1, num_args = 1..)]
		ids: Vec<String>,
	},
	/// Install all packages.
	Install,
	/// List all packages.
	List,
	/// Upgrade all packages.
	Upgrade {
		/// Packages to upgrade, upgrade all if unspecified.
		#[arg(index = 1, num_args = 0..)]
		ids: Vec<String>,
	},
}

#[derive(clap::Args)]
pub(super) struct CommandPub {
	/// Kind of message.
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
	pub(super) fn receiver() -> Result<Id> {
		if let Some(s) = std::env::var("YAZI_PID").ok().filter(|s| !s.is_empty()) {
			Ok(s.parse()?)
		} else {
			bail!("No `YAZI_ID` environment variable found.")
		}
	}
}

#[derive(clap::Args)]
pub(super) struct CommandPubTo {
	/// Receiver ID.
	#[arg(index = 1)]
	pub(super) receiver: Id,
	/// Kind of message.
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
	/// Kind of messages to subscribe to, separated by commas if multiple.
	#[arg(index = 1)]
	pub(super) kinds: String,
}

// --- Macros
macro_rules! impl_emit_body {
	($name:ident) => {
		impl $name {
			#[allow(dead_code)]
			pub(super) fn body(self) -> Result<String> {
				let cmd: Vec<_> = [Either::Left(self.name)]
					.into_iter()
					.chain(self.args.into_iter().map(|s| Either::Right(s.into_encoded_bytes())))
					.collect();
				Ok(serde_json::to_string(&cmd)?)
			}
		}
	};
}

macro_rules! impl_pub_body {
	($name:ident) => {
		impl $name {
			#[allow(dead_code)]
			pub(super) fn body(&self) -> Result<Cow<'_, str>> {
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
