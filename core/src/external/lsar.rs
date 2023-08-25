use std::path::Path;

use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Value;
use shared::PeekError;
use tokio::process::Command;

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

#[allow(clippy::manual_map)]
pub async fn lsar(path: &Path, skip: usize, limit: usize) -> Result<Vec<LsarFile>, PeekError> {
	let output =
		Command::new("lsar").args(["-j", "-jss"]).arg(path).kill_on_drop(true).output().await?;

	if !output.status.success() {
		return Err(String::from_utf8_lossy(&output.stderr).to_string().into());
	}

	#[derive(Deserialize)]
	struct Outer {
		#[serde(rename = "lsarContents")]
		contents: Vec<Value>,
	}

	let output = String::from_utf8_lossy(&output.stdout);
	let contents = serde_json::from_str::<Outer>(output.trim()).map_err(|e| anyhow!(e))?.contents;

	let mut i = 0;
	let mut files = Vec::with_capacity(limit);
	for content in contents {
		i += 1;
		if i > skip + limit {
			break;
		} else if i <= skip {
			continue;
		}

		let attributes = if let Some(p) = content.get("XADPosixPermissions").and_then(|p| p.as_u64()) {
			Some(LsarAttr::Posix(p as u16))
		} else if let Some(a) = content.get("XADWindowsFileAttributes").and_then(|a| a.as_u64()) {
			Some(LsarAttr::Windows(a as u16))
		} else if let Some(a) = content.get("XADDOSFileAttributes").and_then(|a| a.as_u64()) {
			Some(LsarAttr::Dos(a as u8))
		} else {
			None
		};

		let mut file = serde_json::from_value::<LsarFile>(content).map_err(|e| anyhow!(e))?;
		file.attributes = attributes;
		files.push(file);
	}

	if skip > 0 && files.len() < limit {
		Err(PeekError::Exceed(i.saturating_sub(limit)))
	} else {
		Ok(files)
	}
}
