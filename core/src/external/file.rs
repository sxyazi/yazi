use std::collections::BTreeMap;

use anyhow::{bail, Result};
use futures::TryFutureExt;
use shared::{MimeKind, Url};
use tokio::process::Command;
use tracing::error;

async fn _file(files: &[&Url]) -> Result<BTreeMap<Url, String>> {
	if files.is_empty() {
		bail!("no files to get mime types for");
	}

	let output = Command::new("file")
		.args([cfg!(windows).then_some("-b").unwrap_or("-bL"), "--mime-type"])
		.args(files)
		.kill_on_drop(true)
		.output()
		.inspect_err(|e| error!("failed to execute `file`: {}", e))
		.await?;

	let output = String::from_utf8_lossy(&output.stdout);
	let mimes = BTreeMap::from_iter(
		files
			.iter()
			.zip(output.trim().lines())
			.filter(|(_, m)| MimeKind::valid(m))
			.map(|(&f, m)| (f.clone(), m.to_string())),
	);

	if mimes.is_empty() {
		error!("failed to get mime types: {:?}", files);
		bail!("failed to get mime types");
	}
	Ok(mimes)
}

pub async fn file(files: &[impl AsRef<Url>]) -> Result<BTreeMap<Url, String>> {
	_file(&files.iter().map(AsRef::as_ref).collect::<Vec<_>>()).await
}
