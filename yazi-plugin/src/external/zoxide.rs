use std::process::Stdio;

use anyhow::{bail, Result};
use tokio::process::Command;
use yazi_shared::fs::Url;

pub struct ZoxideOpt {
	pub cwd: Url,
	pub query: Option<String>,
}

pub async fn zoxide(opt: ZoxideOpt) -> Result<Url> {
	let child = Command::new("zoxide")
		.args(["query", "--exclude"])
		.arg(&opt.cwd)
		.arg(if let Some(query) = &opt.query { query } else { "--interactive" })
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.spawn()?;

	let output = child.wait_with_output().await?;
	let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();

	if !selected.is_empty() {
		return Ok(Url::from(selected));
	}
	bail!("No match")
}
