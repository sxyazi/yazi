use std::borrow::Cow;

use anyhow::Result;
use tracing::error;
use yazi_macro::time;
use yazi_term::tty::TTY;

use crate::{CLOSE, ESCAPE, Emulator, START, TMUX};

pub struct Mux;

impl Mux {
	pub fn csi(s: &str) -> Cow<'_, str> {
		if TMUX.get() {
			Cow::Owned(format!(
				"{START}{}{CLOSE}",
				s.trim_start_matches('\x1b').replace('\x1b', ESCAPE.get()),
			))
		} else {
			Cow::Borrowed(s)
		}
	}

	pub fn tmux_passthrough() {
		let output = time!(
			"Running `tmux set -p allow-passthrough on`",
			std::process::Command::new("tmux")
				.args(["set", "-p", "allow-passthrough", "on"])
				.stdin(std::process::Stdio::null())
				.stdout(std::process::Stdio::null())
				.stderr(std::process::Stdio::piped())
				.spawn()
				.and_then(|c| c.wait_with_output())
		);

		match output {
			Ok(o) if o.status.success() => {}
			Ok(o) => {
				error!(
					"Running `tmux set -p allow-passthrough on` failed: {:?}, {}",
					o.status,
					String::from_utf8_lossy(&o.stderr)
				);
			}
			Err(e) => {
				error!("Failed to spawn `tmux set -p allow-passthrough on`: {e}");
			}
		}
	}

	pub fn tmux_drain() -> Result<()> {
		if TMUX.get() {
			crossterm::execute!(TTY.writer(), crossterm::style::Print(Mux::csi("\x1b[5n")))?;
			_ = Emulator::read_until_dsr();
		}
		Ok(())
	}

	pub fn tmux_sixel_flag() -> &'static str {
		let stdout = std::process::Command::new("tmux")
			.args(["-LwU0dju1is5", "-f/dev/null", "start", ";", "display", "-p", "#{sixel_support}"])
			.output()
			.ok()
			.and_then(|o| String::from_utf8(o.stdout).ok())
			.unwrap_or_default();

		match stdout.trim() {
			"1" => "Supported",
			"0" => "Unsupported",
			_ => "Unknown",
		}
	}

	pub(super) fn term_program() -> (Option<String>, Option<String>) {
		let (mut term, mut program) = (None, None);
		if !TMUX.get() {
			return (term, program);
		}

		let Ok(output) = time!(
			"Running `tmux show-environment`",
			std::process::Command::new("tmux").arg("show-environment").output()
		) else {
			return (term, program);
		};

		for line in String::from_utf8_lossy(&output.stdout).lines() {
			if let Some((k, v)) = line.trim().split_once('=') {
				match k {
					"TERM" => term = Some(v.to_owned()),
					"TERM_PROGRAM" => program = Some(v.to_owned()),
					_ => continue,
				}
			}
			if term.is_some() && program.is_some() {
				break;
			}
		}
		(term, program)
	}
}
