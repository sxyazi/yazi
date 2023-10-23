use std::{path::Path, sync::Arc};

use regex::Regex;
use tokio::process::Command;
use yazi_adaptor::Image;
use yazi_shared::PeekError;

pub async fn pdftoppm(src: &Path, dest: impl AsRef<Path>, skip: usize) -> Result<(), PeekError> {
	let output = Command::new("pdftoppm")
		.args(["-singlefile", "-jpeg", "-jpegopt", "quality=75", "-f"])
		.arg((skip + 1).to_string())
		.arg(src)
		.kill_on_drop(true)
		.output()
		.await?;

	if !output.status.success() {
		let s = String::from_utf8_lossy(&output.stderr);
		let pages: usize = Regex::new(r"the last page \((\d+)\)")
			.unwrap()
			.captures(&s)
			.map(|cap| cap[1].parse().unwrap())
			.unwrap_or(0);

		return if pages > 0 { Err(PeekError::Exceed(pages - 1)) } else { Err(s.to_string().into()) };
	}

	Ok(Image::precache_anyway(Arc::new(output.stdout), dest).await?)
}
