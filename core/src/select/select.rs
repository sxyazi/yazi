use anyhow::{anyhow, Result};
use ratatui::prelude::Rect;
use shared::tty_size;
use tokio::sync::oneshot::Sender;

use super::SELECT_PADDING;
use crate::Position;

#[derive(Default)]
pub struct Select {
	title:    String,
	items:    Vec<String>,
	position: (u16, u16),

	offset:   usize,
	cursor:   usize,
	callback: Option<Sender<Result<usize>>>,

	pub visible: bool,
}

pub struct SelectOpt {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,
}

impl Select {
	pub fn show(&mut self, opt: SelectOpt, tx: Sender<Result<usize>>) {
		self.close(false);

		self.title = opt.title;
		self.items = opt.items;
		self.position = match opt.position {
			Position::Coords(x, y) => (x, y),
			_ => unimplemented!(),
		};
		self.callback = Some(tx);
		self.visible = true;
	}

	pub fn close(&mut self, submit: bool) -> bool {
		if let Some(cb) = self.callback.take() {
			let _ = cb.send(if submit { Ok(self.cursor) } else { Err(anyhow!("canceled")) });
		}

		self.cursor = 0;
		self.offset = 0;
		self.visible = false;
		true
	}

	pub fn next(&mut self, step: usize) -> bool {
		let len = self.items.len();
		if len == 0 {
			return false;
		}

		let old = self.cursor;
		self.cursor = (self.cursor + step).min(len - 1);

		let limit = self.limit();
		if self.cursor >= len.min(self.offset + limit) {
			self.offset = len.saturating_sub(limit).min(self.offset + self.cursor - old);
		}

		old != self.cursor
	}

	pub fn prev(&mut self, step: usize) -> bool {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(step);

		if self.cursor < self.offset {
			self.offset = self.offset.saturating_sub(old - self.cursor);
		}

		old != self.cursor
	}

	#[inline]
	pub fn window(&self) -> &[String] {
		let end = (self.offset + self.limit()).min(self.items.len());
		&self.items[self.offset..end]
	}

	#[inline]
	pub fn limit(&self) -> usize {
		self.items.len().min(tty_size().ws_row.saturating_sub(SELECT_PADDING).min(5) as usize)
	}
}

impl Select {
	#[inline]
	pub fn title(&self) -> String { self.title.clone() }

	#[inline]
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }

	#[inline]
	pub fn area(&self) -> Rect {
		Rect {
			x:      self.position.0,
			y:      self.position.1 + 2,
			width:  50,
			height: self.limit() as u16 + SELECT_PADDING,
		}
	}
}
