use std::path::Path;

use tokio::process::Command;
use yazi_config::PREVIEW;
use yazi_shared::PeekError;

pub async fn ffmpegthumbnailer(src: &Path, dest: &Path, skip: usize) -> Result<(), PeekError> {
	let percentage = 5 + skip;
	if percentage > 95 {
		return Err(PeekError::Exceed(95 - 5));
	}

	let output = Command::new("ffmpegthumbnailer")
		.arg("-i")
		.arg(src)
		.arg("-o")
		.arg(dest)
		.args(["-t", &percentage.to_string()])
		.args(["-q", "6", "-c", "jpeg", "-s", &PREVIEW.max_width.to_string()])
		.kill_on_drop(true)
		.output()
		.await?;

	if !output.status.success() {
		return Err(String::from_utf8_lossy(&output.stderr).to_string().into());
	}
	Ok(())
}
