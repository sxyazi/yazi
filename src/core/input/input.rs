use anyhow::{anyhow, Result};
use ratatui::layout::Rect;
use tokio::sync::oneshot::Sender;
use unicode_width::UnicodeWidthStr;

use crate::{core::Position, misc::CharKind};

#[derive(Default)]
pub struct Input {
	title:    String,
	value:    String,
	position: (u16, u16),

	op:    InputOp,
	range: Option<(usize, usize)>,

	mode:     InputMode,
	offset:   usize,
	cursor:   usize,
	callback: Option<Sender<Result<String>>>,

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputOp {
	#[default]
	None,
	Delete(bool, bool),
	Yank(bool),
}

impl InputOp {
	#[inline]
	fn set_include(&mut self) {
		match self {
			Self::Delete(_, ref mut b) => *b = true,
			Self::Yank(ref mut b) => *b = true,
			_ => (),
		}
	}
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
		self.offset = self.value.width().saturating_sub(50);
		self.callback = Some(tx);
		self.visible = true;
	}

	pub fn close(&mut self, submit: bool) -> bool {
		if let Some(cb) = self.callback.take() {
			let _ = cb.send(if submit { Ok(self.value.clone()) } else { Err(anyhow!("canceled")) });
		}

		self.op = InputOp::None;
		self.range = None;

		self.mode = InputMode::Insert;
		self.visible = false;
		true
	}

	pub fn escape(&mut self) -> bool {
		match self.mode {
			InputMode::Normal => {
				self.range = None;
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

		self.mode = InputMode::Insert;
		if append {
			self.move_(1);
		}
		true
	}

	pub fn visual(&mut self) -> bool {
		if self.mode != InputMode::Normal {
			return false;
		}

		self.range = Some((self.cursor, self.cursor));
		true
	}

	pub fn move_(&mut self, step: isize) -> bool {
		let old = self.cursor;

		if step <= 0 {
			self.cursor = self.cursor.saturating_sub(step.abs() as usize);
		} else {
			let count = self.count();
			self.cursor += step as usize;

			if self.cursor >= count {
				self.op.set_include();
				self.cursor = if self.mode == InputMode::Insert { count } else { count.saturating_sub(1) };
			}
		}

		if self.cursor != old {
			if self.cursor < self.offset {
				self.offset = self.cursor;
			} else if self.cursor > self.offset + 50 {
				self.offset = self.cursor.saturating_sub(50);
			}
		}

		self.handle_op() || self.cursor != old
	}

	#[inline]
	pub fn move_in_operating(&mut self, step: isize) -> bool {
		if self.op == InputOp::None { false } else { self.move_(step) }
	}

	pub fn backward(&mut self) -> bool {
		if self.cursor == 0 {
			return self.handle_op();
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
			return self.handle_op();
		}
		if end {
			self.op.set_include();
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
			if b {
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
				self.op = InputOp::Delete(insert, false);
				if self.range.is_some() {
					return self.handle_op();
				}

				self.range = Some((self.cursor, self.cursor));
				false
			}
			InputOp::Delete(..) => {
				self.move_(-(self.value.len() as isize));
				self.value.clear();

				self.op = InputOp::None;
				self.range = None;

				self.mode = if insert { InputMode::Insert } else { InputMode::Normal };
				true
			}
			_ => false,
		}
	}

	fn handle_op(&mut self) -> bool {
		if let Some(ref mut range) = self.range {
			*range = (range.0.min(self.cursor), range.0.max(self.cursor));
		}

		match self.op {
			InputOp::None => return false,
			InputOp::Delete(insert, include) => {
				let range = self.range.take().unwrap();
				if !self.value.is_empty() {
					let (start, end) = (self.idx(range.0), self.idx(range.1 + include as usize));
					self.value.drain(start.unwrap()..end.unwrap());
				}
				self.mode = if insert {
					self.cursor = range.0.min(self.count());
					InputMode::Insert
				} else {
					self.cursor = range.0.min(self.count().saturating_sub(1));
					InputMode::Normal
				};
			}
			InputOp::Yank(include) => {}
		}

		self.op = InputOp::None;
		true
	}
}

impl Input {
	#[inline]
	pub fn title(&self) -> String { self.title.clone() }

	#[inline]
	pub fn value(&self) -> String { self.value.clone() }

	#[inline]
	pub fn area(&self) -> Rect {
		Rect { x: self.position.0, y: self.position.1 + 2, width: 50, height: 3 }
	}

	#[inline]
	pub fn mode(&self) -> InputMode { self.mode }

	#[inline]
	pub fn cursor(&self) -> (u16, u16) {
		let width = self
			.value
			.chars()
			.enumerate()
			.take_while(|(i, _)| *i < self.cursor)
			.map(|(_, c)| c)
			.collect::<String>()
			.width() as u16;

		let area = self.area();
		(area.x + width + 1, area.y + 1)
	}

	pub fn range(&self) -> Option<Rect> {
		if let Some((start, end)) = self.range {
			let end = self
				.value
				.chars()
				.skip(start)
				.enumerate()
				.take_while(|(i, _)| *i < end)
				.map(|(_, c)| c)
				.collect::<String>()
				.width() as u16;

			let start = self
				.value
				.chars()
				.enumerate()
				.take_while(|(i, _)| *i < start)
				.map(|(_, c)| c)
				.collect::<String>()
				.width() as u16;

			return Some(Rect {
				x:      self.position.0 + 1 + start,
				y:      self.position.1 + 3,
				width:  end,
				height: 1,
			});
		}
		None
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
}
