use std::{env, ffi::{OsStr, OsString}, process, time::Duration};

use clap::{CommandFactory, Parser};
use clap_complete::{ArgValueCompleter, CompletionCandidate, Shell};
use tokio::{runtime::Handle, task, time::timeout};
use yazi_boot::Args;
use yazi_shared::strand::StrandLike;
use yazi_vfs::Completer;

#[derive(Parser)]
#[command(name = "yazi")]
pub(crate) struct Completion {
	#[arg(long = "completion", value_name = "SHELL")]
	shell: Shell,
	#[arg(long)]
	index: Option<usize>,
	#[arg(last = true, required = true)]
	words: Vec<OsString>,
}

impl Completion {
	pub(crate) fn complete() -> anyhow::Result<()> {
		let Some(arg) = env::args_os().nth(1) else { return Ok(()) };
		if !arg.as_encoded_bytes().starts_with(b"--completion=") {
			return Ok(());
		}

		let Self { shell, index, words } = Self::parse();
		task::block_in_place(|| {
			yazi_shim::clap_complete::Completion::complete(shell, index, words, Self::command)
		})?;

		process::exit(0);
	}

	fn command() -> clap::Command {
		Args::command().mut_arg("entries", |arg| arg.add(ArgValueCompleter::new(Self::entries)))
	}

	fn entries(s: &OsStr) -> Vec<CompletionCandidate> {
		let query = async {
			let _headless = yazi_headless::init().await?;
			let completer = Completer::new(s.as_encoded_bytes());
			let (key, word) = completer.parse()?;

			let mut cands = vec![];
			for item in Completer::complete(&key, &word).await? {
				let Some(value) = completer.resolve(&key, &item).or(item.value) else { continue };
				cands.push(CompletionCandidate::new(value).hide(item.name.starts_with(".")));
			}
			Ok(cands)
		};

		// Exit on timeout to avoid hanging the shell.
		let cands: anyhow::Result<_> = Handle::current()
			.block_on(timeout(Duration::from_secs(5), query))
			.unwrap_or_else(|_| process::exit(0));

		let mut cands = cands.unwrap_or_default();
		cands.sort();
		cands
	}
}
