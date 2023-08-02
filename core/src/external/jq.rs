use std::path::Path;

use anyhow::{bail, Result};
use config::PREVIEW;
use tokio::process::Command;

pub async fn jq(path: &Path) -> Result<String> {
	let output = Command::new("jq")
		.args(["-C", "--indent", &PREVIEW.tab_size.to_string(), "."])
		.arg(path)
		.kill_on_drop(true)
		.output()
		.await?;

	if !output.status.success() {
		bail!("failed to get json: {}", String::from_utf8_lossy(&output.stderr));
	}
	Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
