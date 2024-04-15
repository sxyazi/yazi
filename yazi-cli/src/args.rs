use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub(super) struct Args {
	#[command(subcommand)]
	pub(super) command: Command,
}

#[derive(Subcommand)]
pub(super) enum Command {
	/// Send a message to remote instances.
	Send(CommandSend),
}

#[derive(clap::Args)]
pub(super) struct CommandSend {
	pub(super) message: String,
}
