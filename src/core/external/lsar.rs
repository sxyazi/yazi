use std::path::Path;

use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::Value;
use tokio::process::Command;
use tracing::info;

#[derive(Debug)]
pub enum LsarAttr {
	Posix(u16),
	Windows(u16),
	Dos(u8),
}

#[derive(Debug, Deserialize)]
pub struct LsarFile {
	#[serde(rename = "XADFileName")]
	pub name:             String,
	#[serde(rename = "XADLastModificationDate")]
	pub last_modified:    String,
	#[serde(rename = "XADFileSize")]
	pub size:             Option<usize>,
	#[serde(rename = "XADCompressedSize")]
	pub compressed_size:  Option<usize>,
	#[serde(rename = "XADCompressionName")]
	pub compression_name: Option<String>,

	#[serde(skip)]
	pub attributes: Option<LsarAttr>,
}

pub async fn lsar(path: &Path) -> Result<Vec<LsarFile>> {
	let output =
		Command::new("lsar").args(["-j", "-jss"]).arg(path).kill_on_drop(true).output().await?;

	if !output.status.success() {
		bail!("failed to get json: {}", String::from_utf8_lossy(&output.stderr));
	}

	#[derive(Deserialize)]
	struct Outer {
		#[serde(rename = "lsarContents")]
		contents: Vec<Value>,
	}

	let output = String::from_utf8_lossy(&output.stdout);
	info!("lsar output: {}", output);
	let contents = serde_json::from_str::<Outer>(output.trim())?.contents;

	let mut files = Vec::with_capacity(contents.len());
	for content in contents {
		let mut file = serde_json::from_value::<LsarFile>(content.clone())?;
		if let Some(p) = content.get("XADPosixPermissions").and_then(|p| p.as_u64()) {
			file.attributes = Some(LsarAttr::Posix(p as u16));
		} else if let Some(a) = content.get("XADWindowsFileAttributes").and_then(|a| a.as_u64()) {
			file.attributes = Some(LsarAttr::Windows(a as u16));
		} else if let Some(a) = content.get("XADDOSFileAttributes").and_then(|a| a.as_u64()) {
			file.attributes = Some(LsarAttr::Dos(a as u8));
		}

		files.push(file);
	}
	Ok(files)
}
