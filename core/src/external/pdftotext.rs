use std::path::Path;

use anyhow::{bail, Result};
use tokio::process::Command;

pub async fn pdftotext(path: &Path) -> Result<String> {
	let output = Command::new("pdftotext")
		.args(["-l", "10", "-nopgbrk", "-q", "--"])
		.arg(path)
        .arg("-")
		.kill_on_drop(true)
		.output()
		.await?;

	if !output.status.success() {
		bail!("failed to get json: {}", String::from_utf8_lossy(&output.stderr));
	}
	Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
