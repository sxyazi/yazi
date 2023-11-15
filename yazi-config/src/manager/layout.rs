use anyhow::bail;
use crossterm::terminal::WindowSize;
use ratatui::{prelude::Rect, widgets::{Block, Padding}};
use serde::{Deserialize, Serialize};
use yazi_shared::Term;

use crate::{PREVIEW, THEME};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "Vec<u16>")]
pub struct ManagerLayout {
	pub parent:  u16,
	pub current: u16,
	pub preview: u16,
	pub all:     u16,
}

impl TryFrom<Vec<u16>> for ManagerLayout {
	type Error = anyhow::Error;

	fn try_from(ratio: Vec<u16>) -> Result<Self, Self::Error> {
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
		let (top, right, bottom, left) = THEME.manager.preview_offset;

		let w = (columns * self.preview) as f64 / self.all as f64;
		let w = if w.fract() > 0.5 { w.ceil() as u16 } else { w.floor() as u16 };

		Rect {
			x:      left.saturating_add(columns - w),
			y:      top,
			width:  w.saturating_sub(left + right),
			height: rows.saturating_sub(top + bottom),
		}
	}

	#[inline]
	pub fn preview_height(&self) -> usize { self.preview_rect().height as usize }

	pub fn image_rect(&self) -> Rect {
		let mut rect = self.preview_rect();
		if PREVIEW.max_width == 0 || PREVIEW.max_height == 0 {
			return rect;
		}
		if let Some((w, h)) = Term::ratio() {
			rect.width = rect.width.min((PREVIEW.max_width as f64 / w).ceil() as u16);
			rect.height = rect.height.min((PREVIEW.max_height as f64 / h).ceil() as u16);
		}
		rect
	}

	pub fn folder_rect(&self) -> Rect {
		let WindowSize { columns, rows, .. } = Term::size();

		let offset = THEME.manager.folder_offset;
		Block::default().padding(Padding::new(offset.3, offset.1, offset.0, offset.2)).inner(Rect {
			x:      columns * self.parent / self.all,
			y:      0,
			width:  columns * self.current / self.all,
			height: rows,
		})
	}

	#[inline]
	pub fn folder_height(&self) -> usize { self.folder_rect().height as usize }
}
