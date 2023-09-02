use std::{collections::BTreeMap, ffi::OsStr, path::PathBuf};

use anyhow::{bail, Result};
use futures::TryFutureExt;
use shared::MimeKind;
use tokio::process::Command;
use tracing::error;

pub async fn file(files: &[impl AsRef<OsStr>]) -> Result<BTreeMap<PathBuf, String>> {
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
			.map(|(f, m)| (f.as_ref().into(), m.to_string())),
	);

	if mimes.is_empty() {
		error!("failed to get mime types: {:?}", files.iter().map(AsRef::as_ref).collect::<Vec<_>>());
		bail!("failed to get mime types");
	}
	Ok(mimes)
}
