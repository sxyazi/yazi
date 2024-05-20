use std::{io::Write, path::Path, process::Stdio};

use ansi_to_tui::IntoText;
use anyhow::{bail, Result};
use ratatui::layout::Rect;
use tokio::process::Command;
use yazi_shared::term::Term;

use crate::Adaptor;

pub(super) struct Chafa;

impl Chafa {
	pub(super) async fn image_show(path: &Path, max: Rect) -> Result<Rect> {
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
			.arg(format!("{}x{}", max.width, max.height))
			.arg(path)
			.stdin(Stdio::null())
			.stdout(Stdio::piped())
			.stderr(Stdio::null())
			.kill_on_drop(true)
			.output()
			.await?;

		if !output.status.success() {
			bail!("chafa failed with status: {}", output.status);
		} else if output.stdout.is_empty() {
			bail!("chafa returned no output");
		}

		let lines: Vec<_> = output.stdout.split(|&b| b == b'\n').collect();
		let Ok(Some(first)) = lines[0].into_text().map(|mut t| t.lines.pop()) else {
			bail!("failed to parse chafa output");
		};

		let area = Rect {
			x:      max.x,
			y:      max.y,
			width:  first.spans.into_iter().map(|s| s.content.chars().count() as u16).sum(),
			height: lines.len() as u16,
		};

		Adaptor::shown_store(area);
		Term::move_lock((max.x, max.y), |stderr| {
			for (i, line) in lines.into_iter().enumerate() {
				stderr.write_all(line)?;
				Term::move_to(stderr, max.x, max.y + i as u16 + 1)?;
			}
			Ok(area)
		})
	}

	pub(super) fn image_erase(area: Rect) -> Result<()> {
		let s = " ".repeat(area.width as usize);
		Term::move_lock((0, 0), |stderr| {
			for y in area.top()..area.bottom() {
				Term::move_to(stderr, area.x, y)?;
				write!(stderr, "{s}")?;
			}
			Ok(())
		})
	}
}
