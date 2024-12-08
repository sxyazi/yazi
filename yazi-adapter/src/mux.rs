use crate::{CLOSE, ESCAPE, NVIM, START, TMUX};

pub struct Mux;

impl Mux {
	pub fn csi(s: &str) -> std::borrow::Cow<str> {
		if *TMUX && !*NVIM {
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
		if !*TMUX {
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
