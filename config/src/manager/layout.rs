use anyhow::bail;
use crossterm::terminal::WindowSize;
use ratatui::prelude::Rect;
use serde::Deserialize;
use shared::Term;

use super::{FOLDER_MARGIN, PREVIEW_BORDER, PREVIEW_MARGIN};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(try_from = "Vec<u32>")]
pub struct ManagerLayout {
	pub parent:  u32,
	pub current: u32,
	pub preview: u32,
	pub all:     u32,
}

impl TryFrom<Vec<u32>> for ManagerLayout {
	type Error = anyhow::Error;

	fn try_from(ratio: Vec<u32>) -> Result<Self, Self::Error> {
		if ratio.len() != 3 {
			bail!("invalid layout ratio: {:?}", ratio);
		}
		if ratio.iter().all(|&r| r == 0) {
			bail!("at least one layout ratio must be non-zero: {:?}", ratio);
		}

		Ok(Self {
			parent:  ratio[0],
			current: ratio[1],
			preview: ratio[2],
			all:     ratio[0] + ratio[1] + ratio[2],
		})
	}
}

impl ManagerLayout {
	pub fn preview_rect(&self) -> Rect {
		let WindowSize { columns, rows, .. } = Term::size();

		let width = (columns as u32 * self.preview) as f64 / self.all as f64;
		let width = if width.fract() > 0.5 { width.ceil() as u16 } else { width.floor() as u16 };

		let x = columns.saturating_sub(width);

		Rect {
			x:      x.saturating_add(PREVIEW_BORDER / 2),
			y:      PREVIEW_MARGIN / 2,
			width:  width.saturating_sub(PREVIEW_BORDER),
			height: rows.saturating_sub(PREVIEW_MARGIN),
		}
	}

	#[inline]
	pub fn preview_height(&self) -> usize { self.preview_rect().height as usize }

	pub fn folder_rect(&self) -> Rect {
		let WindowSize { columns, rows, .. } = Term::size();

		Rect {
			x:      (columns as u32 * self.parent / self.all) as u16,
			y:      FOLDER_MARGIN / 2,
			width:  (columns as u32 * self.current / self.all) as u16,
			height: rows.saturating_sub(FOLDER_MARGIN),
		}
	}

	#[inline]
	pub fn folder_height(&self) -> usize { self.folder_rect().height as usize }
}
