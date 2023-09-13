use std::collections::BTreeMap;

use anyhow::{bail, Result};
use futures::TryFutureExt;
use shared::{MimeKind, Url};
use tokio::{fs, io::AsyncReadExt, process::Command};
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
	let _files = &files.iter().map(AsRef::as_ref).collect::<Vec<_>>();
	_file(_files).or_else(move |_| fallback(_files)).await
}

async fn fallback(files: &[&Url]) -> Result<BTreeMap<Url, String>> {
	let mut mimes = BTreeMap::new();
	for &file in files {
		if let Ok(mime) = guess_mime(file).await {
			mimes.insert(file.clone(), mime);
		}
	}

	if mimes.is_empty() {
		error!("failed to get mime types: {:?}", files);
		bail!("failed to get mime types");
	}

	Ok(mimes)
}

async fn guess_mime(file: impl AsRef<Url>) -> Result<String> {
	// First, try to guess mime type from file extenstion.
	// If fail, either return "text/plain" or "application/octet-stream" by
	// trying to convert 1kb of data to utf8 string
	let file = file.as_ref();
	match mime_guess::from_path(file).first() {
		Some(mime) => Ok(mime.to_string()),
		None => {
			let mut buf = [0; 1024];
			let num_bytes = fs::File::open(file).await?.read(&mut buf).await?;
			if String::from_utf8(buf[..num_bytes].to_vec()).is_ok() {
				Ok(String::from("text/plain"))
			} else {
				Ok(String::from("application/octet-stream"))
			}
		}
	}
}
