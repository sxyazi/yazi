use std::ops::Range;

use ratatui::crossterm::cursor::SetCursorStyle;
use unicode_width::UnicodeWidthStr;
use yazi_config::YAZI;
use yazi_plugin::CLIPBOARD;

use super::{InputSnap, InputSnaps, mode::InputMode, op::InputOp};

pub type InputCallback = Box<dyn Fn(&str, &str)>;

#[derive(Default)]
pub struct Input {
	pub snaps:    InputSnaps,
	pub limit:    usize,
	pub callback: Option<InputCallback>,
}

impl Input {
	pub fn new(value: String, limit: usize, callback: InputCallback) -> Self {
		Self { snaps: InputSnaps::new(value, limit), limit, callback: Some(callback) }
	}

	pub(super) fn handle_op(&mut self, cursor: usize, include: bool) -> bool {
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
					futures::executor::block_on(CLIPBOARD.set(&drain));
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
				futures::executor::block_on(CLIPBOARD.set(yanked));
			}
		};

		snap.cursor = snap.count().saturating_sub(snap.mode.delta()).min(snap.cursor);
		if snap == &old {
			return false;
		}
		if !matches!(old.op, InputOp::None | InputOp::Select(_)) {
			self.snaps.tag(self.limit).then(|| self.flush_value());
		}
		true
	}

	pub(super) fn flush_value(&mut self) {
		if let Some(cb) = &self.callback {
			let (before, after) = self.partition();
			cb(before, after);
		}
	}
}

impl Input {
	#[inline]
	pub fn value(&self) -> &str { &self.snap().value }

	#[inline]
	pub fn visible_value(&self) -> &str { self.snap().slice(self.snap().window(self.limit)) }

	#[inline]
	pub fn mode(&self) -> InputMode { self.snap().mode }

	#[inline]
	pub fn cursor(&self) -> u16 {
		let snap = self.snap();
		snap.slice(snap.offset..snap.cursor).width() as u16
	}

	pub fn cursor_shape(&self) -> SetCursorStyle {
		use InputMode as M;

		match self.mode() {
			M::Normal if YAZI.input.cursor_blink => SetCursorStyle::BlinkingBlock,
			M::Normal if !YAZI.input.cursor_blink => SetCursorStyle::SteadyBlock,
			M::Insert if YAZI.input.cursor_blink => SetCursorStyle::BlinkingBar,
			M::Insert if !YAZI.input.cursor_blink => SetCursorStyle::SteadyBar,
			M::Replace if YAZI.input.cursor_blink => SetCursorStyle::BlinkingUnderScore,
			M::Replace if !YAZI.input.cursor_blink => SetCursorStyle::SteadyUnderScore,
			M::Normal | M::Insert | M::Replace => unreachable!(),
		}
	}

	pub fn selected(&self) -> Option<Range<u16>> {
		let snap = self.snap();
		let start = snap.op.start()?;

		let (start, end) =
			if start < snap.cursor { (start, snap.cursor) } else { (snap.cursor + 1, start + 1) };

		let win = snap.window(self.limit);
		let Range { start, end } = start.max(win.start)..end.min(win.end);

		let s = snap.slice(snap.offset..start).width() as u16;
		Some(s..s + snap.slice(start..end).width() as u16)
	}

	#[inline]
	pub fn partition(&self) -> (&str, &str) {
		let snap = self.snap();
		let idx = snap.idx(snap.cursor).unwrap();
		(&snap.value[..idx], &snap.value[idx..])
	}

	#[inline]
	pub fn snap(&self) -> &InputSnap { self.snaps.current() }

	#[inline]
	pub fn snap_mut(&mut self) -> &mut InputSnap { self.snaps.current_mut() }
}
