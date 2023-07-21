use anyhow::{anyhow, Result};
use ratatui::layout::Rect;
use tokio::sync::oneshot::Sender;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::InputSnap;
use crate::{core::{external, Position}, misc::CharKind};

#[derive(Default)]
pub struct Input {
	title:            String,
	pub(crate) value: String,
	position:         (u16, u16),

	pub(crate) op:    InputOp,
	pub(crate) start: Option<usize>,

	pub(crate) mode:   InputMode,
	pub(crate) offset: usize,
	pub(crate) cursor: usize,
	callback:          Option<Sender<Result<String>>>,

	pub visible: bool,
}

pub struct InputOpt {
	pub title:    String,
	pub value:    String,
	pub position: Position,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
	Normal,
	#[default]
	Insert,
}

impl InputMode {
	#[inline]
	fn delta(&self) -> usize { (*self != InputMode::Insert) as usize }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputOp {
	#[default]
	None,
	Delete(bool),
	Yank,
}

impl Input {
	pub fn show(&mut self, opt: InputOpt, tx: Sender<Result<String>>) {
		self.close(false);

		self.title = opt.title;
		self.value = opt.value;
		self.position = match opt.position {
			Position::Coords(x, y) => (x, y),
			_ => unimplemented!(),
		};

		self.cursor = self.count();
		self.offset = self.cursor.saturating_sub(50 - 2 /* Border width */);
		self.callback = Some(tx);
		self.visible = true;
	}

	pub fn close(&mut self, submit: bool) -> bool {
		if let Some(cb) = self.callback.take() {
			let _ = cb.send(if submit { Ok(self.value.clone()) } else { Err(anyhow!("canceled")) });
		}

		self.op = InputOp::None;
		self.start = None;

		self.mode = InputMode::Insert;
		self.visible = false;
		true
	}

	pub fn escape(&mut self) -> bool {
		match self.mode {
			InputMode::Normal => {
				self.op = InputOp::None;
				self.start = None;
			}
			InputMode::Insert => {
				self.mode = InputMode::Normal;
				self.move_(-1);
			}
		}
		true
	}

	pub fn insert(&mut self, append: bool) -> bool {
		if self.mode != InputMode::Normal {
			return false;
		}

		self.op = InputOp::None;
		self.start = None;
		self.mode = InputMode::Insert;
		if append {
			self.move_(1);
		}
		true
	}

	pub fn visual(&mut self) -> bool {
		if self.mode != InputMode::Normal {
			return false;
		} else if self.value.is_empty() {
			return false;
		}

		self.start = Some(self.cursor);
		true
	}

	pub fn move_(&mut self, step: isize) -> bool {
		let b = self.handle_op(
			if step <= 0 {
				self.cursor.saturating_sub(step.abs() as usize)
			} else {
				self.count().min(self.cursor + step as usize)
			},
			false,
		);

		if self.cursor < self.offset {
			self.offset = self.cursor;
		} else if self.cursor >= self.offset + 50 - 2 {
			self.offset = self.cursor.saturating_sub(50 - 2 - self.mode.delta());
		}
		b
	}

	#[inline]
	pub fn move_in_operating(&mut self, step: isize) -> bool {
		if self.op == InputOp::None { false } else { self.move_(step) }
	}

	pub fn backward(&mut self) -> bool {
		if self.cursor == 0 {
			return self.move_(0);
		}

		let idx = self.idx(self.cursor).unwrap_or(self.value.len());
		let mut it = self.value[..idx].chars().rev().enumerate();
		let mut prev = CharKind::new(it.next().unwrap().1);
		for (i, c) in it {
			let c = CharKind::new(c);
			if prev != CharKind::Space && prev != c {
				return self.move_(-(i as isize));
			}
			prev = c;
		}

		if prev != CharKind::Space {
			return self.move_(-(self.value.len() as isize));
		}
		false
	}

	pub fn forward(&mut self, end: bool) -> bool {
		if self.value.is_empty() {
			return self.move_(0);
		}

		let mut it = self.value.chars().skip(self.cursor).enumerate();
		let mut prev = CharKind::new(it.next().unwrap().1);
		for (i, c) in it {
			let c = CharKind::new(c);
			let b = if end {
				prev != CharKind::Space && prev != c && i != 1
			} else {
				c != CharKind::Space && c != prev
			};
			if b && self.op != InputOp::None {
				return self.move_(i as isize);
			} else if b {
				return self.move_(if end { i - 1 } else { i } as isize);
			}
			prev = c;
		}

		self.move_(self.value.len() as isize)
	}

	pub fn type_(&mut self, c: char) -> bool {
		if self.cursor < 1 {
			self.value.insert(0, c);
		} else if self.cursor == self.count() {
			self.value.push(c);
		} else {
			self.value.insert(self.idx(self.cursor).unwrap(), c);
		}
		self.move_(1)
	}

	pub fn backspace(&mut self) -> bool {
		if self.cursor < 1 {
			return false;
		} else if self.cursor == self.count() {
			self.value.pop();
		} else {
			self.value.remove(self.idx(self.cursor - 1).unwrap());
		}
		self.move_(-1)
	}

	pub fn delete(&mut self, insert: bool) -> bool {
		match self.op {
			InputOp::None => {
				self.op = InputOp::Delete(insert);
				if self.start.is_some() {
					return self.handle_op(self.cursor, true).then(|| self.move_(0)).is_some();
				}

				self.start = Some(self.cursor);
				false
			}
			InputOp::Delete(..) => {
				self.move_(-(self.value.len() as isize));
				self.value.clear();
				self.mode = if insert { InputMode::Insert } else { InputMode::Normal };
				true
			}
			_ => false,
		}
	}

	pub fn yank(&mut self) -> bool {
		match self.op {
			InputOp::None => {
				self.op = InputOp::Yank;
				if self.start.is_some() {
					return self.handle_op(self.cursor, true).then(|| self.move_(0)).is_some();
				}

				self.start = Some(self.cursor);
				false
			}
			InputOp::Yank => {
				self.start = Some(0);
				self.move_(self.value.len() as isize);
				false
			}
			_ => false,
		}
	}

	pub fn paste(&mut self, before: bool) -> bool {
		if self.start.is_some() {
			self.op = InputOp::Delete(false);
			self.handle_op(self.cursor, true);
		}

		let str =
			futures::executor::block_on(async { external::clipboard_get().await }).unwrap_or_default();
		if str.is_empty() {
			return false;
		}

		self.insert(!before);
		for c in str.chars() {
			self.type_(c);
		}
		self.escape();
		true
	}

	fn handle_op(&mut self, cursor: usize, include: bool) -> bool {
		let snap = self.snap();
		let range = if self.op == InputOp::None { None } else { self.range(cursor, include) };

		match self.op {
			InputOp::None => {
				self.cursor = cursor;
			}
			InputOp::Delete(insert) => {
				let range = range.unwrap();
				let (start, end) = (self.idx(range.0), self.idx(range.1));

				self.value.drain(start.unwrap()..end.unwrap());
				self.mode = if insert { InputMode::Insert } else { InputMode::Normal };
				self.cursor = range.0;
			}
			InputOp::Yank => {
				let range = range.unwrap();
				let (start, end) = (self.idx(range.0), self.idx(range.1));
				let yanked = &self.value[start.unwrap()..end.unwrap()];

				futures::executor::block_on(async {
					external::clipboard_set(yanked).await.ok();
				});
			}
		};

		self.op = InputOp::None;
		self.cursor = self.count().saturating_sub(self.mode.delta()).min(self.cursor);
		snap != self.snap()
	}
}

impl Input {
	#[inline]
	pub fn title(&self) -> String { self.title.clone() }

	#[inline]
	pub fn value(&self) -> &str {
		let win = self.window();
		&self.value[self.idx(win.0).unwrap()..self.idx(win.1).unwrap()]
	}

	#[inline]
	pub fn mode(&self) -> InputMode { self.mode }

	#[inline]
	pub fn area(&self) -> Rect {
		Rect { x: self.position.0, y: self.position.1 + 2, width: 50, height: 3 }
	}

	#[inline]
	pub fn cursor(&self) -> (u16, u16) {
		let area = self.area();
		let width = self.value[self.offset..self.idx(self.cursor).unwrap()].width() as u16;

		(area.x + width + 1, area.y + 1)
	}

	pub fn selected(&self) -> Option<Rect> {
		if self.start.is_none() {
			return None;
		}

		let start = self.start.unwrap();
		let (start, end) =
			if start < self.cursor { (start, self.cursor) } else { (self.cursor + 1, start + 1) };

		let win = self.window();
		let (start, end) = (start.max(win.0), end.min(win.1));
		let (start, end) = (self.idx(start).unwrap(), self.idx(end).unwrap());

		let offset = self.idx(self.offset).unwrap();
		Some(Rect {
			x:      self.position.0 + 1 + self.value[offset..start].width() as u16,
			y:      self.position.1 + 3,
			width:  self.value[start..end].width() as u16,
			height: 1,
		})
	}

	#[inline]
	fn count(&self) -> usize { self.value.chars().count() }

	#[inline]
	fn idx(&self, n: usize) -> Option<usize> {
		self
			.value
			.char_indices()
			.nth(n)
			.map(|(i, _)| i)
			.or_else(|| if n == self.count() { Some(self.value.len()) } else { None })
	}

	#[inline]
	fn range(&mut self, cursor: usize, include: bool) -> Option<(usize, usize)> {
		self
			.start
			.take()
			.map(|s| if s <= cursor { (s, cursor) } else { (cursor, s) })
			.map(|(s, e)| (s, e + include as usize))
	}

	#[inline]
	fn window(&self) -> (usize, usize) {
		let mut len = 0;
		let v = self
			.value
			.chars()
			.enumerate()
			.skip(self.offset)
			.map_while(|(i, c)| {
				len += c.width().unwrap_or(0);
				if len > 50 - 2/*Border width*/ { None } else { Some(i) }
			})
			.collect::<Vec<_>>();
		(v.first().copied().unwrap_or(0), v.last().map(|l| l + 1).unwrap_or(0))
	}

	#[inline]
	fn snap(&self) -> InputSnap { InputSnap::from(self) }
}
