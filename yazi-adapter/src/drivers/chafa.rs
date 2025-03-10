use std::{io::Write, path::Path, process::Stdio};

use ansi_to_tui::IntoText;
use anyhow::{Result, bail};
use crossterm::{cursor::MoveTo, queue};
use ratatui::layout::Rect;
use tokio::process::Command;

use crate::{Adapter, Emulator};

pub(crate) struct Chafa;

impl Chafa {
	pub(crate) async fn image_show(path: &Path, max: Rect) -> Result<Rect> {
		let child = Command::new("chafa")
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
			.arg(format!("{}x{}", max.width, max.height))
			.arg(path)
			.stdin(Stdio::null())
			.stdout(Stdio::piped())
			.stderr(Stdio::null())
			.kill_on_drop(true)
			.spawn()?;

		let output = child.wait_with_output().await?;
		if !output.status.success() {
			bail!("chafa failed with status: {}", output.status);
		} else if output.stdout.is_empty() {
			bail!("chafa returned no output");
		}

		let lines: Vec<_> = output.stdout.split(|&b| b == b'\n').collect();
		let Ok(Some(first)) = lines[0].to_text().map(|mut t| t.lines.pop()) else {
			bail!("failed to parse chafa output");
		};

		let area = Rect {
			x:      max.x,
			y:      max.y,
			width:  first.width() as u16,
			height: lines.len() as u16,
		};

		Adapter::Chafa.image_hide()?;
		Adapter::shown_store(area);
		Emulator::move_lock((max.x, max.y), |w| {
			for (i, line) in lines.into_iter().enumerate() {
				w.write_all(line)?;
				queue!(w, MoveTo(max.x, max.y + i as u16 + 1))?;
			}
			Ok(area)
		})
	}

	pub(crate) fn image_erase(area: Rect) -> Result<()> {
		let s = " ".repeat(area.width as usize);
		Emulator::move_lock((0, 0), |w| {
			for y in area.top()..area.bottom() {
				queue!(w, MoveTo(area.x, y))?;
				write!(w, "{s}")?;
			}
			Ok(())
		})
	}
}
