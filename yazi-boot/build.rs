#[path = "src/args.rs"]
mod args;

use std::{env, error::Error};

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use vergen_gitcl::{BuildBuilder, Emitter, GitclBuilder};

fn main() -> Result<(), Box<dyn Error>> {
	Emitter::default()
		.add_instructions(&BuildBuilder::default().build_date(true).build()?)?
		.add_instructions(&GitclBuilder::default().commit_date(true).sha(true).build()?)?
		.emit()?;

	if env::var_os("YAZI_GEN_COMPLETIONS").is_none() {
		return Ok(());
	}

	let cmd = &mut args::Args::command();
	let bin = "yazi";
	let out = "completions";

	std::fs::create_dir_all(out)?;
	generate_to(Shell::Bash, cmd, bin, out)?;
	generate_to(Shell::Fish, cmd, bin, out)?;
	generate_to(Shell::Zsh, cmd, bin, out)?;
	generate_to(Shell::Elvish, cmd, bin, out)?;
	generate_to(Shell::PowerShell, cmd, bin, out)?;
	generate_to(clap_complete_nushell::Nushell, cmd, bin, out)?;
	generate_to(clap_complete_fig::Fig, cmd, bin, out)?;

	Ok(())
}
