use std::{env, ffi::OsString, io::{self, BufWriter, Write}};

use anyhow::ensure;
use clap::Command;
use clap_complete::{CompletionCandidate, Shell, engine};

pub struct Completion;

impl Completion {
	pub fn complete<F>(
		shell: Shell,
		index: Option<usize>,
		mut words: Vec<OsString>,
		factory: F,
	) -> anyhow::Result<()>
	where
		F: FnOnce() -> Command,
	{
		ensure!(!words.is_empty(), "Missing completion command word");
		let index = index.unwrap_or(words.len() - 1);
		if index == words.len() {
			words.push(OsString::new());
		}
		ensure!(index < words.len(), "Completion index is out of bounds");

		let mut output = BufWriter::new(io::stdout().lock());
		for c in engine::complete(&mut factory(), words, index, env::current_dir().ok().as_deref())? {
			Self::write(&mut output, shell, c)?;
		}

		output.flush()?;
		Ok(())
	}

	fn write(
		output: &mut impl Write,
		shell: Shell,
		candidate: CompletionCandidate,
	) -> io::Result<()> {
		let Some(value) = candidate.get_value().to_str() else { return Ok(()) };
		if value.contains('\0') {
			return Ok(());
		} else if shell == Shell::Bash {
			return write!(output, "{value}\0");
		} else if value.chars().any(char::is_control) {
			return Ok(());
		}

		let help = candidate.get_help().map(ToString::to_string).unwrap_or_default();
		let help = help.lines().next().unwrap_or_default().replace(char::is_control, " ");
		if shell == Shell::Zsh {
			write!(output, "{}", value.replace('\\', "\\\\").replace(':', "\\:"))?;
			if !help.is_empty() {
				write!(output, ":{}", help.replace('\\', "\\\\"))?;
			}
		} else {
			write!(output, "{value}")?;
			if matches!(shell, Shell::Fish | Shell::PowerShell) && !help.is_empty() {
				write!(output, "\t{help}")?;
			}
		}

		writeln!(output)
	}
}
