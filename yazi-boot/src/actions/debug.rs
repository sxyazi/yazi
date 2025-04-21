use std::{env, ffi::OsStr, fmt::Write};

use regex::Regex;
use yazi_adapter::Mux;
use yazi_shared::timestamp_us;

use super::Actions;

impl Actions {
	pub(super) fn debug() -> Result<String, std::fmt::Error> {
		let mut s = String::new();
		writeln!(s, "\nYazi")?;
		writeln!(s, "    Version: {}", Self::version())?;
		writeln!(s, "    Debug  : {}", cfg!(debug_assertions))?;
		writeln!(s, "    Triple : {}", Self::triple())?;
		writeln!(s, "    Rustc  : {}", Self::rustc())?;

		writeln!(s, "\nYa")?;
		writeln!(s, "    Version: {}", Self::process_output("ya", "--version"))?;

		writeln!(s, "\nEmulator")?;
		writeln!(s, "    TERM                : {:?}", env::var_os("TERM"))?;
		writeln!(s, "    TERM_PROGRAM        : {:?}", env::var_os("TERM_PROGRAM"))?;
		writeln!(s, "    TERM_PROGRAM_VERSION: {:?}", env::var_os("TERM_PROGRAM_VERSION"))?;
		writeln!(s, "    Brand.from_env      : {:?}", yazi_adapter::Brand::from_env())?;
		writeln!(s, "    Emulator.detect     : {:?}", yazi_adapter::EMULATOR)?;

		writeln!(s, "\nAdapter")?;
		writeln!(s, "    Adapter.matches    : {:?}", yazi_adapter::ADAPTOR)?;
		writeln!(s, "    Dimension.available: {:?}", yazi_adapter::Dimension::available())?;

		writeln!(s, "\nDesktop")?;
		writeln!(s, "    XDG_SESSION_TYPE           : {:?}", env::var_os("XDG_SESSION_TYPE"))?;
		writeln!(s, "    WAYLAND_DISPLAY            : {:?}", env::var_os("WAYLAND_DISPLAY"))?;
		writeln!(s, "    DISPLAY                    : {:?}", env::var_os("DISPLAY"))?;
		writeln!(s, "    SWAYSOCK                   : {:?}", env::var_os("SWAYSOCK"))?;
		#[rustfmt::skip]
		writeln!(s, "    HYPRLAND_INSTANCE_SIGNATURE: {:?}", env::var_os("HYPRLAND_INSTANCE_SIGNATURE"))?;
		writeln!(s, "    WAYFIRE_SOCKET             : {:?}", env::var_os("WAYFIRE_SOCKET"))?;

		writeln!(s, "\nSSH")?;
		writeln!(s, "    shared.in_ssh_connection: {:?}", yazi_shared::in_ssh_connection())?;

		writeln!(s, "\nWSL")?;
		writeln!(s, "    WSL: {:?}", yazi_adapter::WSL)?;

		writeln!(s, "\nVariables")?;
		writeln!(s, "    SHELL           : {:?}", env::var_os("SHELL"))?;
		writeln!(s, "    EDITOR          : {:?}", env::var_os("EDITOR"))?;
		writeln!(s, "    VISUAL          : {:?}", env::var_os("VISUAL"))?;
		writeln!(s, "    YAZI_FILE_ONE   : {:?}", env::var_os("YAZI_FILE_ONE"))?;
		writeln!(s, "    YAZI_CONFIG_HOME: {:?}", env::var_os("YAZI_CONFIG_HOME"))?;
		writeln!(s, "    YAZI_ZOXIDE_OPTS: {:?}", env::var_os("YAZI_ZOXIDE_OPTS"))?;
		writeln!(s, "    FZF_DEFAULT_OPTS: {:?}", env::var_os("FZF_DEFAULT_OPTS"))?;

		writeln!(s, "\nText Opener")?;
		writeln!(
			s,
			"    default     : {:?}",
			yazi_config::YAZI.opener.first(yazi_config::YAZI.open.all("f75a.txt", "text/plain"))
		)?;
		writeln!(
			s,
			"    block-create: {:?}",
			yazi_config::YAZI.opener.block(yazi_config::YAZI.open.all("bulk-create.txt", "text/plain"))
		)?;
		writeln!(
			s,
			"    block-rename: {:?}",
			yazi_config::YAZI.opener.block(yazi_config::YAZI.open.all("bulk-rename.txt", "text/plain"))
		)?;

		writeln!(s, "\nMultiplexers")?;
		writeln!(s, "    TMUX               : {}", yazi_adapter::TMUX)?;
		writeln!(s, "    tmux version       : {}", Self::process_output("tmux", "-V"))?;
		writeln!(s, "    tmux build flags   : enable-sixel={}", Mux::tmux_sixel_flag())?;
		writeln!(s, "    ZELLIJ_SESSION_NAME: {:?}", env::var_os("ZELLIJ_SESSION_NAME"))?;
		writeln!(s, "    Zellij version     : {}", Self::process_output("zellij", "--version"))?;

		writeln!(s, "\nDependencies")?;
		#[rustfmt::skip]
		writeln!(s, "    file          : {}", Self::process_output(env::var_os("YAZI_FILE_ONE").unwrap_or("file".into()), "--version"))?;
		writeln!(s, "    ueberzugpp    : {}", Self::process_output("ueberzugpp", "--version"))?;
		#[rustfmt::skip]
		writeln!(s, "    ffmpeg/ffprobe: {} / {}", Self::process_output("ffmpeg", "-version"), Self::process_output("ffprobe", "-version"))?;
		writeln!(s, "    pdftoppm      : {}", Self::process_output("pdftoppm", "--help"))?;
		writeln!(s, "    magick        : {}", Self::process_output("magick", "--version"))?;
		writeln!(s, "    fzf           : {}", Self::process_output("fzf", "--version"))?;
		#[rustfmt::skip]
		writeln!(s, "    fd/fdfind     : {} / {}", Self::process_output("fd", "--version"), Self::process_output("fdfind", "--version"))?;
		writeln!(s, "    rg            : {}", Self::process_output("rg", "--version"))?;
		writeln!(s, "    chafa         : {}", Self::process_output("chafa", "--version"))?;
		writeln!(s, "    zoxide        : {}", Self::process_output("zoxide", "--version"))?;
		#[rustfmt::skip]
		writeln!(s, "    7zz/7z        : {} / {}", Self::process_output("7zz", "i"), Self::process_output("7z", "i"))?;
		writeln!(s, "    resvg         : {}", Self::process_output("resvg", "--version"))?;
		writeln!(s, "    jq            : {}", Self::process_output("jq", "--version"))?;

		writeln!(s, "\nClipboard")?;
		#[rustfmt::skip]
		writeln!(s, "    wl-copy/paste: {} / {}", Self::process_output("wl-copy", "--version"), Self::process_output("wl-paste", "--version"))?;
		writeln!(s, "    xclip        : {}", Self::process_output("xclip", "-version"))?;
		writeln!(s, "    xsel         : {}", Self::process_output("xsel", "--version"))?;

		writeln!(s, "\nRoutine")?;
		writeln!(s, "    `file -bL --mime-type`: {}", Self::file1_output())?;

		writeln!(
			s,
			"\n\nSee https://yazi-rs.github.io/docs/plugins/overview#debugging on how to enable logging or debug runtime errors."
		)?;

		Ok(s)
	}

	fn process_output(name: impl AsRef<OsStr>, arg: impl AsRef<OsStr>) -> String {
		match std::process::Command::new(&name).arg(arg).output() {
			Ok(out) if out.status.success() => {
				let line =
					String::from_utf8_lossy(&if out.stdout.is_empty() { out.stderr } else { out.stdout })
						.trim()
						.lines()
						.next()
						.unwrap_or_default()
						.to_owned();
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

	fn file1_output() -> String {
		use std::io::Write;

		let p = env::temp_dir().join(format!("yazi-debug-{}", timestamp_us()));
		std::fs::File::create_new(&p).map(|mut f| f.write_all(b"Hello, World!")).ok();

		let cmd = env::var_os("YAZI_FILE_ONE").unwrap_or("file".into());
		match std::process::Command::new(cmd).args(["-bL", "--mime-type"]).arg(&p).output() {
			Ok(out) => {
				String::from_utf8_lossy(&out.stdout).trim().lines().next().unwrap_or_default().to_owned()
			}
			Err(e) => format!("{e}"),
		}
	}
}
