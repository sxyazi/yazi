use std::{env, ffi::OsString, fs, path::{Path, PathBuf}, process::Command};

use anyhow::{Context, Result, bail, ensure};

pub(super) fn cargo() -> OsString { env::var_os("CARGO").unwrap_or_else(|| "cargo".into()) }

pub(super) fn cargo_bin_dir() -> Result<PathBuf> {
	if let Some(root) = env::var_os("CARGO_INSTALL_ROOT") {
		return Ok(PathBuf::from(root).join("bin"));
	}
	if let Some(home) = cargo_home_dir() {
		return Ok(home.join("bin"));
	}
	bail!("failed to determine the Cargo bin directory")
}

fn cargo_home_dir() -> Option<PathBuf> {
	env::var_os("CARGO_HOME").map(PathBuf::from).or_else(|| {
		env::var_os("HOME")
			.or_else(|| env::var_os("USERPROFILE"))
			.map(|home| PathBuf::from(home).join(".cargo"))
	})
}

pub(super) fn workspace_root() -> Result<PathBuf> {
	Path::new(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.map(Path::to_owned)
		.context("yazi-build must be inside the Yazi workspace")
}

pub(super) fn is_linux_target(target: &str) -> bool {
	target.contains("-linux-") || (target.is_empty() && cfg!(target_os = "linux"))
}

pub(super) fn is_windows_target(target: &str) -> bool {
	target.contains("-windows-") || (target.is_empty() && cfg!(windows))
}

pub(super) fn copy_bins(target: &str, profile: &str, to: &Path) -> Result<()> {
	let mut from = workspace_root()?.join("target");
	if !target.is_empty() {
		from.push(target);
	}
	from.push(profile);

	let ext = if is_windows_target(target) { ".exe" } else { "" };
	copy_file(&from.join(format!("yazi{ext}")), &to.join(format!("yazi{ext}")))?;
	copy_file(&from.join(format!("ya{ext}")), &to.join(format!("ya{ext}")))?;
	Ok(())
}

pub(super) fn copy_file(from: &Path, to: &Path) -> Result<()> {
	fs::copy(from, to)
		.map(|_| ())
		.with_context(|| format!("failed to copy {} to {}", from.display(), to.display()))
}

pub(super) fn run(cmd: &mut Command) -> Result<()> {
	let status = cmd
		.current_dir(workspace_root()?)
		.status()
		.with_context(|| format!("failed to spawn {cmd:?}"))?;

	ensure!(status.success(), "{cmd:?} exited with status {status}");
	Ok(())
}
