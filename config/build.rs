#[path = "src/boot/cli.rs"]
mod cli;

use std::{fs, io};

use clap::CommandFactory;
use clap_complete::{generate_to, Shell::*};
use clap_complete_fig::Fig;
use clap_complete_nushell::Nushell;

use self::cli::Args;

fn main() -> io::Result<()> {
	let cmd = &mut Args::command();
	let name = "yazi";
	let dir = "completions";

	fs::create_dir_all(dir)?;

	generate_to(Bash, cmd, name, dir)?;
	generate_to(Fish, cmd, name, dir)?;
	generate_to(Zsh, cmd, name, dir)?;
	generate_to(Elvish, cmd, name, dir)?;
	generate_to(PowerShell, cmd, name, dir)?;
	generate_to(Nushell, cmd, name, dir)?;
	generate_to(Fig, cmd, name, dir)?;

	Ok(())
}
