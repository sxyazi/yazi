use std::{path::Path, process::Stdio};

use anyhow::{bail, Result};
use imagesize::ImageSize;
use ratatui::layout::Rect;
use tokio::process::Command;

use crate::Image;

pub(super) struct Chafa;

impl Chafa {
	pub(super) async fn image_show(path: &Path, rect: Rect) -> Result<(u32, u32)> {
		let p = path.to_owned();
		let ImageSize { width: w, height: h } =
			tokio::task::spawn_blocking(move || imagesize::size(p)).await??;

		let output = Command::new("chafa")
			.args([
				"-f",
				"symbols",
				"--relative",
				"off",
				"--polite",
				"on",
				"--passthrough",
				"none",
				"--animate",
				"off",
				"--view-size",
			])
			.arg(format!("{}x{}", rect.width, rect.height))
			.arg(path)
			.stdin(Stdio::null())
			.stderr(Stdio::null())
			.stdout(Stdio::piped())
			.kill_on_drop(true)
			.output()
			.await?;

		if !output.status.success() {
			bail!("chafa failed with status: {}", output.status);
		}

		// output.stdout

		let (max_w, max_h) = Image::max_pixel(rect);
		if w <= max_w as usize && h <= max_h as usize {
			return Ok((w as u32, h as u32));
		}

		let ratio = f64::min(max_w as f64 / w as f64, max_h as f64 / h as f64);
		Ok(((w as f64 * ratio).round() as u32, (h as f64 * ratio).round() as u32))
	}

	pub(super) fn image_erase(_: Rect) -> Result<()> {
		if let Some(tx) = &*DEMON {
			Ok(tx.send(None)?)
		} else {
			bail!("uninitialized ueberzugpp");
		}
	}
}
