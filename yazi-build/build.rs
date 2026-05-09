use std::{env, fs, io::Write, path::{Path, PathBuf}, process::{self, Command, Stdio}, time::{SystemTime, UNIX_EPOCH}};

use anyhow::{Context, Result, bail, ensure};
use yazi_tty::TTY;

fn main() -> Result<()> {
	yazi_tty::init();

	let manifest = env::var_os("CARGO_MANIFEST_DIR")
		.context("missing CARGO_MANIFEST_DIR")?
		.to_string_lossy()
		.replace(r"\", "/");

	let rev = if manifest.contains("/registry/src/index.crates.io-") {
		Some("shipped")
	} else if manifest.contains("/git/checkouts/yazi-") {
		None
	} else {
		println!("cargo::warning=Unexpected manifest dir: {manifest}");
		return Ok(());
	};

	let os = env::var("CARGO_CFG_TARGET_OS").context("missing CARGO_CFG_TARGET_OS")?;
	let tmp = temp_repo_dir()?;

	TTY.writer().write_all(b"\nCloning Yazi repository...\n")?;
	clone_repo(&tmp, rev).context("Failed to clone the Yazi repository")?;

	TTY.writer().write_all(b"\nBuilding Yazi binaries...\n")?;
	build_repo(&tmp, &os).context("Failed to build Yazi from the cloned repository")?;

	TTY.writer().write_all(b"\nInstalling yazi and ya into cargo bin...\n")?;
	install_bins(&tmp, &os).context("Failed to install `yazi` and `ya` into cargo bin")?;

	Ok(())
}

fn temp_repo_dir() -> Result<PathBuf> {
	let nonce = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|d| d.as_nanos())
		.context("Time went backwards")?;

	Ok(env::temp_dir().join(format!("yazi-build-{}-{nonce}", process::id())))
}

fn clone_repo(tmp: &Path, rev: Option<&str>) -> Result<()> {
	let mut cmd = Command::new("git");
	cmd.args(["-c", "advice.detachedHead=false", "clone", "--depth", "1"]);

	if let Some(rev) = rev {
		cmd.args(["--branch", rev]);
	}

	run_streamed(cmd.arg("https://github.com/sxyazi/yazi.git").arg(tmp))
}

fn build_repo(tmp: &Path, target_os: &str) -> Result<()> {
	let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());

	let mut cmd = Command::new(cargo);
	cmd.current_dir(tmp).arg("build").env("CARGO_TARGET_DIR", "target").arg("--locked");

	if target_os == "windows" {
		cmd.args(["--profile", "release-windows"]);
	} else {
		cmd.arg("--release");
	}

	run_streamed(&mut cmd)
}

fn install_bins(tmp: &Path, target_os: &str) -> Result<()> {
	let profile = if target_os == "windows" { "release-windows" } else { "release" };
	let ext = if target_os == "windows" { ".exe" } else { "" };
	let bin_dir = cargo_bin_dir()?;

	fs::create_dir_all(&bin_dir)?;
	install_bin(
		&tmp.join("target").join(profile).join(format!("yazi{ext}")),
		&bin_dir.join(format!("yazi{ext}")),
	)?;
	install_bin(
		&tmp.join("target").join(profile).join(format!("ya{ext}")),
		&bin_dir.join(format!("ya{ext}")),
	)?;

	Ok(())
}

fn install_bin(from: &Path, to: &Path) -> Result<()> {
	ensure!(from.is_file(), "Built binary not found: {}", from.display());

	if to.exists() {
		fs::remove_file(to)
			.with_context(|| format!("failed to remove existing binary: {}", to.display()))?;
	}

	fs::copy(from, to)
		.with_context(|| format!("failed to copy {} to {}", from.display(), to.display()))?;
	fs::set_permissions(to, fs::metadata(from)?.permissions())
		.with_context(|| format!("failed to preserve permissions on {}", to.display()))?;
	Ok(())
}

fn cargo_bin_dir() -> Result<PathBuf> {
	if let Some(root) = env::var_os("CARGO_INSTALL_ROOT") {
		return Ok(PathBuf::from(root).join("bin"));
	}
	if let Some(home) = env::var_os("CARGO_HOME") {
		return Ok(PathBuf::from(home).join("bin"));
	}
	if let Some(home) = env::var_os("HOME") {
		return Ok(PathBuf::from(home).join(".cargo/bin"));
	}
	if let Some(home) = env::var_os("USERPROFILE") {
		return Ok(PathBuf::from(home).join(".cargo/bin"));
	}
	bail!("Failed to determine cargo bin directory")
}

fn run_streamed(cmd: &mut Command) -> Result<()> {
	let stdin = {
		let input = TTY.lockin();
		Stdio::from(input.try_clone()?)
	};

	let (stdout, stderr) = {
		let mut output = TTY.lockout();
		output.flush()?;

		(Stdio::from(output.get_ref().try_clone()?), Stdio::from(output.get_ref().try_clone()?))
	};

	let status =
		cmd.stdin(stdin).stdout(stdout).stderr(stderr).status().context("failed to spawn process")?;

	ensure!(status.success(), "process exited with status {status}");
	Ok(())
}
