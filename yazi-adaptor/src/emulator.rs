use std::{env, io::{stderr, LineWriter, Read}};

use anyhow::{anyhow, Result};
use crossterm::{cursor::{RestorePosition, SavePosition}, execute, style::Print, terminal::{disable_raw_mode, enable_raw_mode}};
use tracing::warn;
use yazi_shared::env_exists;

use crate::{Adaptor, TMUX};

#[derive(Clone, Debug)]
pub enum Emulator {
	Unknown(Vec<Adaptor>),
	Kitty,
	Konsole,
	Iterm2,
	WezTerm,
	Foot,
	Ghostty,
	BlackBox,
	VSCode,
	Tabby,
	Hyper,
	Mintty,
	Neovim,
	Apple,
}

impl Emulator {
	pub fn adapters(self) -> Vec<Adaptor> {
		match self {
			Self::Unknown(adapters) => adapters,
			Self::Kitty => vec![Adaptor::Kitty],
			Self::Konsole => vec![Adaptor::KittyOld, Adaptor::Iterm2, Adaptor::Sixel],
			Self::Iterm2 => vec![Adaptor::Iterm2, Adaptor::Sixel],
			Self::WezTerm => vec![Adaptor::Iterm2, Adaptor::Sixel],
			Self::Foot => vec![Adaptor::Sixel],
			Self::Ghostty => vec![Adaptor::KittyOld],
			Self::BlackBox => vec![Adaptor::Sixel],
			Self::VSCode => vec![Adaptor::Iterm2, Adaptor::Sixel],
			Self::Tabby => vec![Adaptor::Iterm2, Adaptor::Sixel],
			Self::Hyper => vec![Adaptor::Iterm2, Adaptor::Sixel],
			Self::Mintty => vec![Adaptor::Iterm2],
			Self::Neovim => vec![],
			Self::Apple => vec![],
		}
	}
}

impl Emulator {
	pub fn detect() -> Self {
		if env_exists("NVIM_LOG_FILE") && env_exists("NVIM") {
			return Self::Neovim;
		}

		let vars = [
			("KITTY_WINDOW_ID", Self::Kitty),
			("KONSOLE_VERSION", Self::Konsole),
			("ITERM_SESSION_ID", Self::Iterm2),
			("WEZTERM_EXECUTABLE", Self::WezTerm),
			("GHOSTTY_RESOURCES_DIR", Self::Ghostty),
			("VSCODE_INJECTION", Self::VSCode),
			("TABBY_CONFIG_DIRECTORY", Self::Tabby),
		];
		match vars.into_iter().find(|v| env_exists(v.0)) {
			Some(var) => return var.1,
			None => warn!("[Adaptor] No special environment variables detected"),
		}

		let (term, program) = Self::via_env();
		match program.as_str() {
			"iTerm.app" => return Self::Iterm2,
			"WezTerm" => return Self::WezTerm,
			"ghostty" => return Self::Ghostty,
			"BlackBox" => return Self::BlackBox,
			"vscode" => return Self::VSCode,
			"Tabby" => return Self::Tabby,
			"Hyper" => return Self::Hyper,
			"mintty" => return Self::Mintty,
			"Apple_Terminal" => return Self::Apple,
			_ => warn!("[Adaptor] Unknown TERM_PROGRAM: {program}"),
		}
		match term.as_str() {
			"xterm-kitty" => return Self::Kitty,
			"foot" => return Self::Foot,
			"foot-extra" => return Self::Foot,
			"xterm-ghostty" => return Self::Ghostty,
			_ => warn!("[Adaptor] Unknown TERM: {term}"),
		}

		Self::via_csi().unwrap_or(Self::Unknown(vec![]))
	}

	pub fn via_env() -> (String, String) {
		fn tmux_env(name: &str) -> Result<String> {
			let output = std::process::Command::new("tmux").args(["show-environment", name]).output()?;

			String::from_utf8(output.stdout)?
				.trim()
				.strip_prefix(&format!("{name}="))
				.map_or_else(|| Err(anyhow!("")), |s| Ok(s.to_string()))
		}

		let mut term = env::var("TERM").unwrap_or_default();
		let mut program = env::var("TERM_PROGRAM").unwrap_or_default();

		if *TMUX {
			term = tmux_env("TERM").unwrap_or(term);
			program = tmux_env("TERM_PROGRAM").unwrap_or(program);
		}

		(term, program)
	}

	pub fn via_csi() -> Result<Self> {
		enable_raw_mode()?;
		execute!(
			LineWriter::new(stderr()),
			SavePosition,
			Print("\x1b[>q\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c"),
			RestorePosition
		)?;

		let mut stdin = std::io::stdin().lock();
		let mut buf = String::with_capacity(200);
		loop {
			let mut c = [0; 1];
			if stdin.read(&mut c)? == 0 {
				break;
			}
			if c[0] == b'c' && buf.contains("\x1b[?") {
				break;
			}
			buf.push(c[0] as char);
		}

		disable_raw_mode().ok();
		let names = [
			("kitty", Self::Kitty),
			("Konsole", Self::Konsole),
			("iTerm2", Self::Iterm2),
			("WezTerm", Self::WezTerm),
			("foot", Self::Foot),
			("ghostty", Self::Ghostty),
		];

		for (name, emulator) in names.iter() {
			if buf.contains(name) {
				return Ok(emulator.clone());
			}
		}

		let mut adapters = Vec::with_capacity(2);
		if buf.contains("\x1b_Gi=31;OK") {
			adapters.push(Adaptor::KittyOld);
		}
		if ["?4;", "?4c", ";4;", ";4c"].iter().any(|s| buf.contains(s)) {
			adapters.push(Adaptor::Sixel);
		}

		Ok(Self::Unknown(adapters))
	}
}
