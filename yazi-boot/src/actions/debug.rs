use std::{env, ffi::OsStr, fmt::Write};

use regex::Regex;
use yazi_adapter::Mux;
use yazi_shared::Xdg;

use super::Actions;

impl Actions {
	pub(super) fn debug() -> Result<String, std::fmt::Error> {
		use std::env::consts::{ARCH, FAMILY, OS};
		let mut s = String::new();

		writeln!(s, "\nYazi")?;
		writeln!(s, "    Version: {}", Self::version())?;
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
		let vars = [
			"XDG_SESSION_TYPE",
			"WAYLAND_DISPLAY",
			"DISPLAY",
			"SWAYLOCK",
			"HYPRLAND_INSTANCE_SIGNATURE",
			"WAYFIRE_SOCKET",
		];
		let width = vars.iter().map(|x| x.len()).max().unwrap();
		for var in &vars {
			writeln!(s, "    {:width$}: {:?}", var, env::var_os(var))?;
		}

		writeln!(s, "\nSSH")?;
		writeln!(s, "    shared.in_ssh_connection: {:?}", yazi_shared::in_ssh_connection())?;

		writeln!(s, "\nWSL")?;
		writeln!(s, "    WSL: {:?}", *yazi_adapter::WSL)?;

		writeln!(s, "\nVariables")?;
		let vars = ["SHELL", "EDITOR", "VISUAL", "YAZI_FILE_ONE", "YAZI_CONFIG_HOME"];
		let width = vars.iter().map(|x| x.len()).max().unwrap();
		for var in &vars {
			writeln!(s, "    {:width$}: {:?}", var, env::var_os(var))?;
		}

		writeln!(s, "\nText Opener")?;
		writeln!(
			s,
			"    default     : {:?}",
			yazi_config::OPEN.openers("f75a.txt", "text/plain").and_then(|a| a.first().cloned())
		)?;
		writeln!(
			s,
			"    block-create: {:?}",
			yazi_config::OPEN.block_opener("bulk-create.txt", "text/plain")
		)?;
		writeln!(
			s,
			"    block-rename: {:?}",
			yazi_config::OPEN.block_opener("bulk-rename.txt", "text/plain")
		)?;

		writeln!(s, "\nMultiplexers")?;
		writeln!(s, "    TMUX               : {:?}", *yazi_adapter::TMUX)?;
		writeln!(s, "    tmux version       : {}", Self::process_output("tmux", "-V"))?;
		writeln!(s, "    tmux build flags   : enable-sixel={}", Mux::tmux_sixel_flag())?;
		writeln!(s, "    ZELLIJ_SESSION_NAME: {:?}", env::var_os("ZELLIJ_SESSION_NAME"))?;
		writeln!(s, "    Zellij version     : {}", Self::process_output("zellij", "--version"))?;

		writeln!(s, "\nDependencies")?;
		let depends = [
			("ueberzugpp", "--version"),
			("ffmpegthumbnailer", "-v"),
			("magick", "--version"),
			("fzf", "--version"),
			("fd", "--version"),
			("rg", "--version"),
			("chafa", "--version"),
			("zoxide", "--version"),
			("7z", "i"),
			("7zz", "i"),
			("jq", "--version"),
		];
		let width = depends.iter().map(|(cmd, _)| cmd.len()).max().unwrap();
		writeln!(
			s,
			"    {:width$}: {}",
			"file",
			Self::process_output(env::var_os("YAZI_FILE_ONE").unwrap_or("file".into()), "--version")
		)?;
		for (cmd, arg) in &depends {
			writeln!(s, "    {:width$}: {}", cmd, Self::process_output(cmd, arg))?;
		}

		writeln!(s, "\n\n--------------------------------------------------")?;
		writeln!(
			s,
			"When reporting a bug, please also upload the `yazi.log` log file - only upload the most recent content by time."
		)?;
		writeln!(s, "You can find it in the {:?} directory.", Xdg::state_dir())?;

		Ok(s)
	}

	fn process_output(name: impl AsRef<OsStr>, arg: impl AsRef<OsStr>) -> String {
		match std::process::Command::new(&name).arg(arg).output() {
			Ok(out) if out.status.success() => {
				let line =
					String::from_utf8_lossy(&out.stdout).trim().lines().next().unwrap_or_default().to_owned();
				if name.as_ref() == "ya" {
					line.trim_start_matches("Ya ").to_owned()
				} else {
					Regex::new(r"\d+\.\d+(\.\d+-\d+|\.\d+|\b)")
						.unwrap()
						.find(&line)
						.map(|m| m.as_str().to_owned())
						.unwrap_or(line)
				}
			}
			Ok(out) => format!("{:?}, {:?}", out.status, String::from_utf8_lossy(&out.stderr)),
			Err(e) => format!("{e}"),
		}
	}
}
