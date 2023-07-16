use std::path::Path;

use anyhow::{bail, Result};
use tokio::process::Command;

use crate::config::PREVIEW;

pub async fn ffmpegthumbnailer(path: &Path, dest: &Path) -> Result<()> {
	let output = Command::new("ffmpegthumbnailer")
		.arg("-i")
		.arg(path)
		.arg("-o")
		.arg(dest)
		.args(["-q", "6", "-c", "jpeg", "-s", &PREVIEW.max_width.to_string()])
		.kill_on_drop(true)
		.output()
		.await?;

	if !output.status.success() {
		bail!("failed to generate video thumbnail: {}", String::from_utf8_lossy(&output.stderr));
	}
	Ok(())
}
