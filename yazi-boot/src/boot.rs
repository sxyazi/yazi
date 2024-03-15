use std::{env, ffi::OsString, path::{Path, PathBuf}, process};

use clap::Parser;
use serde::Serialize;
use yazi_config::PREVIEW;
use yazi_shared::{fs::{current_cwd, expand_path}, Xdg};

use super::Args;
use crate::ARGS;

#[derive(Debug, Serialize)]
pub struct Boot {
	pub cwd:  PathBuf,
	pub file: Option<OsString>,

	pub config_dir: PathBuf,
	pub flavor_dir: PathBuf,
	pub plugin_dir: PathBuf,
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

	fn action_version() {
		println!(
			"yazi {} ({} {})",
			env!("CARGO_PKG_VERSION"),
			env!("VERGEN_GIT_SHA"),
			env!("VERGEN_BUILD_DATE")
		);
	}

	fn action_debug() {
		print!("Yazi\n    ");
		Self::action_version();

		println!("\nEnvironment");
		println!(
			"    OS: {}-{} ({})",
			std::env::consts::OS,
			std::env::consts::ARCH,
			std::env::consts::FAMILY
		);
		println!("    Debug: {}", cfg!(debug_assertions));

		println!("\nEmulator");
		println!("    Emulator.via_env: {:?}", yazi_adaptor::Emulator::via_env());
		println!("    Emulator.via_csi: {:?}", yazi_adaptor::Emulator::via_csi());
		println!("    Emulator.detect: {:?}", yazi_adaptor::Emulator::detect());

		println!("\nAdaptor");
		println!("    Adaptor.matches: {:?}", yazi_adaptor::Adaptor::matches());

		println!("\ntmux");
		println!("    TMUX: {:?}", *yazi_adaptor::TMUX);

		println!("\nZellij");
		println!("    ZELLIJ_SESSION_NAME: {:?}", env::var_os("ZELLIJ_SESSION_NAME"));

		println!("\nDesktop");
		println!("    XDG_SESSION_TYPE: {:?}", env::var_os("XDG_SESSION_TYPE"));
		println!("    WAYLAND_DISPLAY: {:?}", env::var_os("WAYLAND_DISPLAY"));
		println!("    DISPLAY: {:?}", env::var_os("DISPLAY"));

		println!("\nUeberzug++");
		println!(
			"    Version: {:?}",
			std::process::Command::new("ueberzugpp").arg("--version").output()
		);

		println!("\nWSL");
		println!(
			"    /proc/sys/fs/binfmt_misc/WSLInterop: {:?}",
			std::fs::symlink_metadata("/proc/sys/fs/binfmt_misc/WSLInterop").is_ok()
		);

		println!("\n\n--------------------------------------------------");
		println!(
			"When reporting a bug, please also upload the `yazi.log` log file - only upload the most recent content by time."
		);
		println!("You can find it in the {:?} directory.", Xdg::state_dir());
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

		let boot = Self {
			cwd,
			file,

			flavor_dir: config_dir.join("flavors"),
			plugin_dir: config_dir.join("plugins"),
			config_dir,
		};

		std::fs::create_dir_all(&boot.flavor_dir).expect("Failed to create flavor directory");
		std::fs::create_dir_all(&boot.plugin_dir).expect("Failed to create plugin directory");
		boot
	}
}

impl Default for Args {
	fn default() -> Self {
		let args = Self::parse();

		if args.debug {
			Boot::action_debug();
			process::exit(0);
		}

		if args.version {
			Boot::action_version();
			process::exit(0);
		}

		if args.clear_cache {
			Boot::action_clear_cache();
			process::exit(0);
		}

		args
	}
}
