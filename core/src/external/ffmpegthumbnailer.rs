use std::path::Path;

use config::PREVIEW;
use shared::PagedError;
use tokio::process::Command;

pub async fn ffmpegthumbnailer(src: &Path, dest: &Path, skip: usize) -> Result<(), PagedError> {
	let percentage = 5 + skip;
	if percentage >= 100 {
		return Err(PagedError::Exceed(94));
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
