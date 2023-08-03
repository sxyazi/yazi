use std::path::Path;

use anyhow::{bail, Result};
use config::PREVIEW;
use tokio::process::Command;

pub async fn ffmpegthumbnailer(src: &Path, dest: &Path) -> Result<()> {
	let output = Command::new("ffmpegthumbnailer")
		.arg("-i")
		.arg(src)
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
