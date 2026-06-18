use std::fmt::{self, Display};

use anyhow::Result;
use tracing::error;
use yazi_macro::{time, writef};
use yazi_shim::cell::SyncCell;
use yazi_tty::{TTY, sequence::RequestDeviceStatus};

use crate::Emulator;

pub static TMUX: SyncCell<bool> = SyncCell::new(false);
pub static ESCAPE: SyncCell<&'static str> = SyncCell::new("\x1b");
pub static START: SyncCell<&'static str> = SyncCell::new("\x1b");
pub static CLOSE: SyncCell<&'static str> = SyncCell::new("");

pub struct Mux;

impl Mux {
	pub fn wrap<T: Display>(s: T) -> impl Display {
		struct Wrapper<T>(T);

		impl<T: Display> Display for Wrapper<T> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				if !TMUX.get() {
					return self.0.fmt(f);
				}

				write!(
					f,
					"{START}{}{CLOSE}",
					self.0.to_string().trim_start_matches('\x1b').replace('\x1b', ESCAPE.get())
				)
			}
		}

		Wrapper(s)
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
			writef!(TTY.writer(), "{}", Self::wrap(RequestDeviceStatus))?;
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
