use std::ops::Range;

use anyhow::{anyhow, Result};
use ratatui::layout::Rect;
use shared::CharKind;
use tokio::sync::oneshot::Sender;
use unicode_width::UnicodeWidthStr;

use super::{mode::InputMode, op::InputOp, InputOpt, InputSnap, InputSnaps};
use crate::{external, Position};

#[derive(Default)]
pub struct Input {
	snaps:       InputSnaps,
	pub visible: bool,

	title:    String,
	position: (u16, u16),
	callback: Option<Sender<Result<String>>>,

	// Shell
	pub(super) highlight: bool,
}

impl Input {
	pub fn show(&mut self, opt: InputOpt, tx: Sender<Result<String>>) {
		self.close(false);
		self.snaps.reset(opt.value);
		self.visible = true;

		self.title = opt.title;
		self.position = match opt.position {
			Position::Coords(x, y) => (x, y),
			_ => unreachable!(),
		};
		self.callback = Some(tx);

		// Shell
		self.highlight = opt.highlight;
	}

	pub fn close(&mut self, submit: bool) -> bool {
		if let Some(cb) = self.callback.take() {
			let _ =
				cb.send(if submit { Ok(self.snap_mut().value.clone()) } else { Err(anyhow!("canceled")) });
		}

		self.visible = false;
		true
	}

	pub fn escape(&mut self) -> bool {
		let snap = self.snap_mut();
		match snap.mode {
			InputMode::Normal if snap.op == InputOp::None => {
				self.close(false);
			}
			InputMode::Normal => {
				snap.op = InputOp::None;
			}
			InputMode::Insert => {
				snap.mode = InputMode::Normal;
				self.move_(-1);
			}
		}
		self.snaps.tag();
		true
	}

	pub fn insert(&mut self, append: bool) -> bool {
		if !self.snap_mut().insert() {
			return false;
		}
		if append {
			self.move_(1);
		}
		true
	}

	#[inline]
	pub fn visual(&mut self) -> bool { self.snap_mut().visual() }

	#[inline]
	pub fn undo(&mut self) -> bool {
		if !self.snaps.undo() {
			return false;
		}
		if self.snap().mode == InputMode::Insert {
			self.escape();
		}
		true
	}

	#[inline]
	pub fn redo(&mut self) -> bool {
		if !self.snaps.redo() {
			return false;
		}
		true
	}

	pub fn move_(&mut self, step: isize) -> bool {
		let snap = self.snap();
		let b = self.handle_op(
			if step <= 0 {
				snap.cursor.saturating_sub(step.abs() as usize)
			} else {
				snap.count().min(snap.cursor + step as usize)
			},
			false,
		);

		let snap = self.snap_mut();
		if snap.cursor < snap.offset {
			snap.offset = snap.cursor;
		} else if snap.value.is_empty() {
			snap.offset = 0;
		} else {
			let delta = snap.mode.delta();
			let s = snap.slice(snap.offset..snap.cursor + delta);
			if s.width() >= /*TODO: hardcode*/ 50 - 2 {
				let s = s.chars().rev().collect::<String>();
				snap.offset = snap.cursor - InputSnap::find_window(&s, 0).end.saturating_sub(delta);
			}
		}

		b
	}

	#[inline]
	pub fn move_in_operating(&mut self, step: isize) -> bool {
		if self.snap_mut().op == InputOp::None { false } else { self.move_(step) }
	}

	pub fn backward(&mut self) -> bool {
		let snap = self.snap();
		if snap.cursor == 0 {
			return self.move_(0);
		}

		let idx = snap.idx(snap.cursor).unwrap_or(snap.len());
		let mut it = snap.value[..idx].chars().rev().enumerate();
		let mut prev = CharKind::new(it.next().unwrap().1);
		for (i, c) in it {
			let c = CharKind::new(c);
			if prev != CharKind::Space && prev != c {
				return self.move_(-(i as isize));
			}
			prev = c;
		}

		if prev != CharKind::Space {
			return self.move_(-(snap.len() as isize));
		}
		false
	}

	pub fn forward(&mut self, end: bool) -> bool {
		let snap = self.snap();
		if snap.value.is_empty() {
			return self.move_(0);
		}

		let mut it = snap.value.chars().skip(snap.cursor).enumerate();
		let mut prev = CharKind::new(it.next().unwrap().1);
		for (i, c) in it {
			let c = CharKind::new(c);
			let b = if end {
				prev != CharKind::Space && prev != c && i != 1
			} else {
				c != CharKind::Space && c != prev
			};
			if b && !matches!(snap.op, InputOp::None | InputOp::Select(_)) {
				return self.move_(i as isize);
			} else if b {
				return self.move_(if end { i - 1 } else { i } as isize);
			}
			prev = c;
		}

		self.move_(snap.len() as isize)
	}

	#[inline]
	pub fn type_(&mut self, c: char) -> bool {
		let mut bits = [0; 4];
		self.type_str(c.encode_utf8(&mut bits))
	}

	pub fn type_str(&mut self, s: &str) -> bool {
		let snap = self.snap_mut();
		if snap.cursor < 1 {
			snap.value.insert_str(0, s);
		} else {
			snap.value.insert_str(snap.idx(snap.cursor).unwrap(), s);
		}
		self.move_(s.chars().count() as isize)
	}

	pub fn backspace(&mut self) -> bool {
		let snap = self.snap_mut();
		if snap.cursor < 1 {
			return false;
		} else {
			snap.value.remove(snap.idx(snap.cursor - 1).unwrap());
		}
		self.move_(-1)
	}

	pub fn delete(&mut self, cut: bool, insert: bool) -> bool {
		match self.snap().op {
			InputOp::None => {
				self.snap_mut().op = InputOp::Delete(cut, insert, self.snap().cursor);
				false
			}
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Delete(cut, insert, start);
				return self.handle_op(self.snap().cursor, true).then(|| self.move_(0)).is_some();
			}
			InputOp::Delete(..) => {
				self.snap_mut().op = InputOp::Delete(cut, insert, 0);
				return self.move_(self.snap().len() as isize);
			}
			_ => false,
		}
	}

	pub fn yank(&mut self) -> bool {
		match self.snap().op {
			InputOp::None => {
				self.snap_mut().op = InputOp::Yank(self.snap().cursor);
				false
			}
			InputOp::Select(start) => {
				self.snap_mut().op = InputOp::Yank(start);
				return self.handle_op(self.snap().cursor, true).then(|| self.move_(0)).is_some();
			}
			InputOp::Yank(_) => {
				self.snap_mut().op = InputOp::Yank(0);
				self.move_(self.snap().len() as isize);
				false
			}
			_ => false,
		}
	}

	pub fn paste(&mut self, before: bool) -> bool {
		if let Some(start) = self.snap().op.start() {
			self.snap_mut().op = InputOp::Delete(false, false, start);
			self.handle_op(self.snap().cursor, true);
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
		let old = self.snap().clone();
		let snap = self.snap_mut();

		match snap.op {
			InputOp::None | InputOp::Select(_) => {
				snap.cursor = cursor;
			}
			InputOp::Delete(cut, insert, _) => {
				let range = snap.op.range(cursor, include).unwrap();
				let Range { start, end } = snap.idx(range.start)..snap.idx(range.end);

				let drain = snap.value.drain(start.unwrap()..end.unwrap()).collect::<String>();
				if cut {
					futures::executor::block_on(async { external::clipboard_set(&drain).await.ok() });
				}

				snap.op = InputOp::None;
				snap.mode = if insert { InputMode::Insert } else { InputMode::Normal };
				snap.cursor = range.start;
			}
			InputOp::Yank(_) => {
				let range = snap.op.range(cursor, include).unwrap();
				let Range { start, end } = snap.idx(range.start)..snap.idx(range.end);
				let yanked = &snap.value[start.unwrap()..end.unwrap()];

				snap.op = InputOp::None;
				futures::executor::block_on(async { external::clipboard_set(yanked).await.ok() });
			}
		};

		snap.cursor = snap.count().saturating_sub(snap.mode.delta()).min(snap.cursor);
		if *snap == old {
			return false;
		}
		if !matches!(old.op, InputOp::None | InputOp::Select(_)) {
			self.snaps.tag();
		}
		true
	}
}

impl Input {
	#[inline]
	pub fn title(&self) -> String { self.title.clone() }

	#[inline]
	pub fn value(&self) -> &str { self.snap().slice(self.snap().window()) }

	#[inline]
	pub fn mode(&self) -> InputMode { self.snap().mode }

	#[inline]
	pub fn area(&self) -> Rect {
		// TODO: hardcode
		Rect { x: self.position.0, y: self.position.1 + 2, width: 50, height: 3 }
	}

	#[inline]
	pub fn cursor(&self) -> (u16, u16) {
		let snap = self.snap();
		let width = snap.slice(snap.offset..snap.cursor).width() as u16;

		let area = self.area();
		(area.x + width + 1, area.y + 1)
	}

	pub fn selected(&self) -> Option<Rect> {
		let snap = self.snap();
		let start = if let Some(s) = snap.op.start() {
			s
		} else {
			return None;
		};

		let (start, end) =
			if start < snap.cursor { (start, snap.cursor) } else { (snap.cursor + 1, start + 1) };

		let win = snap.window();
		let Range { start, end } = start.max(win.start)..end.min(win.end);

		Some(Rect {
			x:      self.position.0 + 1 + snap.slice(snap.offset..start).width() as u16,
			y:      self.position.1 + 3,
			width:  snap.slice(start..end).width() as u16,
			height: 1,
		})
	}

	#[inline]
	fn snap(&self) -> &InputSnap { self.snaps.current() }

	#[inline]
	fn snap_mut(&mut self) -> &mut InputSnap { self.snaps.current_mut() }
}
