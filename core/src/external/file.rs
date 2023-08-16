use std::{collections::BTreeMap, ffi::OsStr, path::PathBuf};

use anyhow::{bail, Result};
use shared::MimeKind;
use tokio::process::Command;

pub async fn file(files: &[impl AsRef<OsStr>]) -> Result<BTreeMap<PathBuf, String>> {
	if files.is_empty() {
		bail!("no files to get mime types for");
	}

	let output = Command::new("file")
		.args(["-b", "--mime-type"])
		.args(files)
		.kill_on_drop(true)
		.output()
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
		bail!("failed to get mime types");
	}
	Ok(mimes)
}
