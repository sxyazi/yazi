use std::{fmt::{self, Display}, process::Stdio};

use tokio::{process::Command, time::{Duration, timeout}};
use tracing::error;

use crate::EMULATOR;

pub const ESCAPE: MuxSequence = MuxSequence("\x1b", "\x1b\x1b");
pub const START: MuxSequence = MuxSequence("\x1b", "\x1bPtmux;\x1b\x1b");
pub const CLOSE: MuxSequence = MuxSequence("", "\x1b\\");

pub struct Mux;

impl Mux {
	pub async fn tmux_passthrough() {
		let output = Command::new("tmux")
			.args(["set", "-p", "allow-passthrough", "on"])
			.kill_on_drop(true)
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::piped())
			.output();

		match timeout(Duration::from_secs(5), output).await {
			Ok(Ok(o)) if o.status.success() => {}
			Ok(Ok(o)) => {
				error!(
					"Running `tmux set -p allow-passthrough on` failed: {:?}, {}",
					o.status,
					String::from_utf8_lossy(&o.stderr)
				);
			}
			Ok(Err(e)) => error!("Failed to spawn `tmux set -p allow-passthrough on`: {e}"),
			Err(_) => error!("Running `tmux set -p allow-passthrough on` timed out"),
		}
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
}

// --- MuxSequence
pub struct MuxSequence(&'static str, &'static str);

impl Display for MuxSequence {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(if EMULATOR.load().tmux { self.1 } else { self.0 })
	}
}
