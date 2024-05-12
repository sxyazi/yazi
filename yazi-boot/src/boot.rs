use std::{collections::HashSet, env, ffi::{OsStr, OsString}, fmt::Write, path::{Path, PathBuf}, process};

use clap::Parser;
use regex::Regex;
use serde::Serialize;
use yazi_config::PREVIEW;
use yazi_shared::{fs::{current_cwd, expand_path}, Xdg};

use super::Args;
use crate::ARGS;

#[derive(Debug, Serialize)]
pub struct Boot {
	pub cwd:  PathBuf,
	pub file: Option<OsString>,

	pub local_events:  HashSet<String>,
	pub remote_events: HashSet<String>,

	pub config_dir: PathBuf,
	pub flavor_dir: PathBuf,
	pub plugin_dir: PathBuf,
	pub state_dir:  PathBuf,
}

impl Boot {
	fn parse_entry(entry: Option<&Path>) -> (PathBuf, Option<OsString>) {
		let entry = match entry {
			Some(p) => expand_path(p),
			None => return (current_cwd().unwrap(), None),
		};

		let parent = entry.parent();
		if parent.is_none() || entry.is_dir() {
			return (entry, None);
		}

		(parent.unwrap().to_owned(), Some(entry.file_name().unwrap().to_owned()))
	}

	fn process_output(name: impl AsRef<OsStr>, arg: impl AsRef<OsStr>) -> String {
		match std::process::Command::new(name.as_ref()).arg(arg).output() {
			Ok(out) if out.status.success() => {
				let line =
					String::from_utf8_lossy(&out.stdout).trim().lines().next().unwrap_or_default().to_owned();
				Regex::new(r"\d+\.\d+(\.\d+-\d+|\.\d+|\b)")
					.unwrap()
					.find(&line)
					.map(|m| m.as_str().to_owned())
					.unwrap_or(line)
			}
			Ok(out) => format!("{:?}, {:?}", out.status, String::from_utf8_lossy(&out.stderr)),
			Err(e) => format!("{e}"),
		}
	}

	fn action_version() -> String {
		format!(
			"{} ({} {})",
			env!("CARGO_PKG_VERSION"),
			env!("VERGEN_GIT_SHA"),
			env!("VERGEN_BUILD_DATE")
		)
	}

	fn action_debug() -> Result<String, std::fmt::Error> {
		use std::env::consts::{ARCH, FAMILY, OS};
		let mut s = String::new();

		writeln!(s, "\nYazi")?;
		writeln!(s, "    Version: {}", Self::action_version())?;
		writeln!(s, "    Debug  : {}", cfg!(debug_assertions))?;
		writeln!(s, "    OS     : {}-{} ({})", OS, ARCH, FAMILY)?;

		writeln!(s, "\nYa")?;
		writeln!(s, "    Version: {}", Self::process_output("ya", "--version"))?;

		writeln!(s, "\nEmulator")?;
		writeln!(s, "    Emulator.via_env: {:?}", yazi_adapter::Emulator::via_env())?;
		writeln!(s, "    Emulator.via_csi: {:?}", yazi_adapter::Emulator::via_csi())?;
		writeln!(s, "    Emulator.detect : {:?}", yazi_adapter::Emulator::detect())?;

		writeln!(s, "\nAdapter")?;
		writeln!(s, "    Adapter.matches: {:?}", yazi_adapter::Adapter::matches())?;

		writeln!(s, "\nDesktop")?;
		writeln!(s, "    XDG_SESSION_TYPE: {:?}", env::var_os("XDG_SESSION_TYPE"))?;
		writeln!(s, "    WAYLAND_DISPLAY : {:?}", env::var_os("WAYLAND_DISPLAY"))?;
		writeln!(s, "    DISPLAY         : {:?}", env::var_os("DISPLAY"))?;

		writeln!(s, "\nSSH")?;
		writeln!(s, "    shared.in_ssh_connection: {:?}", yazi_shared::in_ssh_connection())?;

		writeln!(s, "\nWSL")?;
		writeln!(
			s,
			"    /proc/sys/fs/binfmt_misc/WSLInterop: {:?}",
			std::fs::symlink_metadata("/proc/sys/fs/binfmt_misc/WSLInterop").is_ok()
		)?;

		writeln!(s, "\nVariables")?;
		writeln!(s, "    SHELL              : {:?}", env::var_os("SHELL"))?;
		writeln!(s, "    EDITOR             : {:?}", env::var_os("EDITOR"))?;
		writeln!(s, "    ZELLIJ_SESSION_NAME: {:?}", env::var_os("ZELLIJ_SESSION_NAME"))?;
		writeln!(s, "    YAZI_FILE_ONE      : {:?}", env::var_os("YAZI_FILE_ONE"))?;
		writeln!(s, "    YAZI_CONFIG_HOME   : {:?}", env::var_os("YAZI_CONFIG_HOME"))?;

		writeln!(s, "\nText Opener")?;
		writeln!(
			s,
			"    default: {:?}",
			yazi_config::OPEN.openers("f75a.txt", "text/plain").and_then(|a| a.first().cloned())
		)?;
		writeln!(s, "    block  : {:?}", yazi_config::OPEN.block_opener("bulk.txt", "text/plain"))?;

		writeln!(s, "\ntmux")?;
		writeln!(s, "    TMUX   : {:?}", *yazi_adapter::TMUX)?;
		writeln!(s, "    Version: {}", Self::process_output("tmux", "-V"))?;

		writeln!(s, "\nDependencies")?;
		writeln!(
			s,
			"    file             : {}",
			Self::process_output(env::var_os("YAZI_FILE_ONE").unwrap_or("file".into()), "--version")
		)?;
		writeln!(s, "    ueberzugpp       : {}", Self::process_output("ueberzugpp", "--version"))?;
		writeln!(s, "    ffmpegthumbnailer: {}", Self::process_output("ffmpegthumbnailer", "-v"))?;
		writeln!(s, "    magick           : {}", Self::process_output("magick", "--version"))?;
		writeln!(s, "    fzf              : {}", Self::process_output("fzf", "--version"))?;
		writeln!(s, "    fd               : {}", Self::process_output("fd", "--version"))?;
		writeln!(s, "    rg               : {}", Self::process_output("rg", "--version"))?;
		writeln!(s, "    chafa            : {}", Self::process_output("chafa", "--version"))?;
		writeln!(s, "    zoxide           : {}", Self::process_output("zoxide", "--version"))?;
		writeln!(s, "    unar             : {}", Self::process_output("unar", "--version"))?;
		writeln!(s, "    jq               : {}", Self::process_output("jq", "--version"))?;

		writeln!(s, "\n\n--------------------------------------------------")?;
		writeln!(
			s,
			"When reporting a bug, please also upload the `yazi.log` log file - only upload the most recent content by time."
		)?;
		writeln!(s, "You can find it in the {:?} directory.", Xdg::state_dir())?;

		Ok(s)
	}

	fn action_clear_cache() {
		if PREVIEW.cache_dir == Xdg::cache_dir() {
			println!("Clearing cache directory: \n{:?}", PREVIEW.cache_dir);
			std::fs::remove_dir_all(&PREVIEW.cache_dir).unwrap();
		} else {
			println!(
				"You've changed the default cache directory, for your data's safety, please clear it manually: \n{:?}",
				PREVIEW.cache_dir
			);
		}
	}
}

impl Default for Boot {
	fn default() -> Self {
		let config_dir = Xdg::config_dir();
		let (cwd, file) = Self::parse_entry(ARGS.entry.as_deref());

		let local_events = ARGS
			.local_events
			.as_ref()
			.map(|s| s.split(',').map(|s| s.to_owned()).collect())
			.unwrap_or_default();
		let remote_events = ARGS
			.remote_events
			.as_ref()
			.map(|s| s.split(',').map(|s| s.to_owned()).collect())
			.unwrap_or_default();

		Self {
			cwd,
			file,

			local_events,
			remote_events,

			flavor_dir: config_dir.join("flavors"),
			plugin_dir: config_dir.join("plugins"),
			config_dir,
			state_dir: Xdg::state_dir(),
		}
	}
}

impl Default for Args {
	fn default() -> Self {
		let args = Self::parse();

		if args.debug {
			println!("{}", Boot::action_debug().unwrap());
			process::exit(0);
		}

		if args.version {
			println!("Yazi {}", Boot::action_version());
			process::exit(0);
		}

		if args.clear_cache {
			Boot::action_clear_cache();
			process::exit(0);
		}

		args
	}
}
