use std::path::Path;

use adaptor::Image;
use anyhow::{bail, Result};
use tokio::process::Command;

pub async fn pdftoppm(src: &Path, dest: impl AsRef<Path>) -> Result<()> {
	let output = Command::new("pdftoppm")
		.args(["-singlefile", "-jpeg", "-jpegopt", "quality=75"])
		.arg(src)
		.kill_on_drop(true)
		.output()
		.await?;

	if !output.status.success() {
		bail!("failed to generate PDF thumbnail: {}", String::from_utf8_lossy(&output.stderr));
	}
	Image::precache_anyway(output.stdout.into(), dest).await
}
