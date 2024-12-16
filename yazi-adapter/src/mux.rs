use anyhow::Result;
use tracing::error;
use yazi_shared::env_exists;

use crate::{CLOSE, ESCAPE, Emulator, NVIM, START, TMUX};

pub struct Mux;

impl Mux {
	pub fn csi(s: &str) -> std::borrow::Cow<str> {
		if *TMUX == 2 && !*NVIM {
			std::borrow::Cow::Owned(format!(
				"{}{}{}",
				*START,
				s.trim_start_matches('\x1b').replace('\x1b', *ESCAPE),
				*CLOSE
			))
		} else {
			std::borrow::Cow::Borrowed(s)
		}
	}

	pub fn tmux_passthrough() -> u8 {
		if !env_exists("TMUX_PANE") || !env_exists("TMUX") {
			return 0;
		}

		let child = std::process::Command::new("tmux")
			.args(["set", "-p", "allow-passthrough", "on"])
			.stdin(std::process::Stdio::null())
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::piped())
			.spawn();

		match child.and_then(|c| c.wait_with_output()) {
			Ok(output) if output.status.success() => return 2,
			Ok(output) => {
				error!(
					"Running `tmux set -p allow-passthrough on` failed: {:?}, {}",
					output.status,
					String::from_utf8_lossy(&output.stderr)
				);
			}
			Err(err) => {
				error!("Failed to spawn `tmux set -p allow-passthrough on`: {err}");
			}
		}
		1
	}

	pub fn tmux_drain() -> Result<()> {
		if *TMUX == 2 && !*NVIM {
			crossterm::execute!(std::io::stderr(), crossterm::style::Print(Mux::csi("\x1b[5n")))?;
			_ = futures::executor::block_on(Emulator::read_until_dsr());
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
		if *TMUX == 0 {
			return (term, program);
		}
		let Ok(output) = std::process::Command::new("tmux").arg("show-environment").output() else {
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
