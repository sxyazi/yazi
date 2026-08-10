use std::{env, io::Write, path::{Path, PathBuf}, process::{self, Command, Stdio}, time::{SystemTime, UNIX_EPOCH}};

use anyhow::{Context, Result, ensure};
use yazi_tty::TTY;

#[allow(dead_code)]
#[path = "src/common.rs"]
mod common;

fn main() -> Result<()> {
	if env::var_os("YAZI_BUILD_BOOTSTRAPPED").is_some_and(|value| value == "1") {
		return Ok(());
	}

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
		println!("cargo::warning=yazi-build installer skipped for manifest dir: {manifest}");
		return Ok(());
	};

	let tmp = temp_repo_dir()?;
	let bin_dir = common::cargo_bin_dir()?;

	TTY.writer().write_all(b"\nCloning Yazi repository...\n")?;
	clone_repo(&tmp, rev).context("Failed to clone the Yazi repository")?;

	TTY.writer().write_all(b"\nBuilding and installing Yazi binaries...\n")?;
	install_repo(&tmp, &bin_dir).context("Failed to install Yazi from the cloned repository")?;

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

fn install_repo(tmp: &Path, bin_dir: &Path) -> Result<()> {
	let mut cmd = Command::new(common::cargo());
	cmd
		.current_dir(tmp)
		.env("YAZI_BUILD_BOOTSTRAPPED", "1")
		.env("CARGO_TARGET_DIR", "target")
		.args(["run", "--locked", "--package", "yazi-build", "--", "install", "--bin-dir"])
		.arg(bin_dir);

	run_streamed(&mut cmd)
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
