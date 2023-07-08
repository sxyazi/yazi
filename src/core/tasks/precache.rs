use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use image::imageops::FilterType;
use tokio::{fs, process::Command};

use crate::{config::PREVIEW, misc::valid_mimetype};

#[derive(Default)]
pub struct Precache;

impl Precache {
	pub fn cache(path: &Path) -> PathBuf {
		PathBuf::from(format!("/tmp/yazi/{:x}", md5::compute(path.to_string_lossy().as_bytes())))
	}

	pub async fn mimetype(files: &Vec<PathBuf>) -> Result<Vec<Option<String>>> {
		if files.is_empty() {
			return Ok(vec![]);
		}

		let output = Command::new("file")
			.args(["-bL", "--mime-type"])
			.args(files)
			.kill_on_drop(true)
			.output()
			.await?;

		if !output.status.success() {
			bail!("failed to get mimetype: {}", String::from_utf8_lossy(&output.stderr));
		}

		Ok(
			String::from_utf8_lossy(&output.stdout)
				.trim()
				.lines()
				.map(|s| if valid_mimetype(s) { Some(s.to_string()) } else { None })
				.collect(),
		)
	}

	pub async fn json(path: &Path) -> Result<String> {
		let output = Command::new("jq")
			.args(["-C", "--indent", &PREVIEW.tab_size.to_string(), "."])
			.arg(path)
			.kill_on_drop(true)
			.output()
			.await?;

		if !output.status.success() {
			bail!("failed to get json: {}", String::from_utf8_lossy(&output.stderr));
		}
		Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
	}

	pub async fn image(path: &Path) -> Result<()> {
		let cache = Self::cache(path);
		if cache.exists() {
			return Ok(());
		}

		let img = image::load_from_memory(&fs::read(path).await?)?;
		let (w, h) = (PREVIEW.max_width, PREVIEW.max_height);

		let img = if img.width() > w || img.height() > h {
			img.resize(w, h, FilterType::Triangle)
		} else {
			img
		};
		Ok(img.save(cache)?)
	}

	pub async fn video(path: &Path) -> Result<()> {
		let cache = Self::cache(path);
		if cache.exists() {
			return Ok(());
		}

		let output = Command::new("ffmpegthumbnailer")
			.arg("-i")
			.arg(path)
			.arg("-o")
			.arg(cache)
			.args(["-q", "6", "-c", "jpeg", "-s", &PREVIEW.max_width.to_string()])
			.kill_on_drop(true)
			.output()
			.await?;

		if !output.status.success() {
			bail!("failed to generate video thumbnail: {}", String::from_utf8_lossy(&output.stderr));
		}
		Ok(())
	}
}
