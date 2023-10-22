use anyhow::bail;
use crossterm::terminal::WindowSize;
use ratatui::{prelude::Rect, widgets::{Block, Padding}};
use serde::{Deserialize, Serialize};
use yazi_shared::Term;

use crate::THEME;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
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

		let offset = THEME.manager.preview_offset;
		Block::default().padding(Padding::new(offset.3, offset.1, offset.0, offset.2)).inner(Rect {
			x: columns.saturating_sub(width),
			y: 0,
			width,
			height: rows,
		})
	}

	#[inline]
	pub fn preview_height(&self) -> usize { self.preview_rect().height as usize }

	pub fn folder_rect(&self) -> Rect {
		let WindowSize { columns, rows, .. } = Term::size();

		let offset = THEME.manager.folder_offset;
		Block::default().padding(Padding::new(offset.3, offset.1, offset.0, offset.2)).inner(Rect {
			x:      (columns as u32 * self.parent / self.all) as u16,
			y:      0,
			width:  (columns as u32 * self.current / self.all) as u16,
			height: rows,
		})
	}

	#[inline]
	pub fn folder_height(&self) -> usize { self.folder_rect().height as usize }
}
