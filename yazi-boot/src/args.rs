use std::path::PathBuf;

use clap::{Parser, builder::{StringValueParser, TypedValueParser}};
use hashbrown::HashSet;
use yazi_shared::{id::Id, strand::StrandBuf};

#[derive(Debug, Default, Parser)]
#[command(name = "yazi")]
#[command(after_help = "See https://yazi-rs.github.io/docs/quick-start for a quick starter.")]
pub struct Args {
	/// Set the current working entry
	#[arg(index = 1, num_args = 1..=9)]
	pub entries: Vec<StrandBuf>,

	/// Write the cwd on exit to this file
	#[arg(long)]
	pub cwd_file:     Option<PathBuf>,
	/// Write the selected files to this file on open fired
	#[arg(long)]
	pub chooser_file: Option<PathBuf>,

	/// Use the specified client ID, must be a globally unique number
	#[arg(long)]
	pub(crate) client_id: Option<Id>,
	/// Report the specified local events to stdout
	#[arg(long, default_value = "", hide_default_value = true, value_parser = events())]
	pub local_events:     HashSet<String>,
	/// Report the specified remote events to stdout
	#[arg(long, default_value = "", hide_default_value = true, value_parser = events())]
	pub remote_events:    HashSet<String>,

	/// Print version
	#[arg(short = 'V', long)]
	pub(crate) version: bool,
}

fn events() -> impl TypedValueParser<Value = HashSet<String>> {
	StringValueParser::new()
		.map(|s| s.split(',').filter(|s| !s.is_empty()).map(str::to_owned).collect())
}
