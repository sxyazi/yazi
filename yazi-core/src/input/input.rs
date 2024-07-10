use std::ops::Range;

use tokio::sync::mpsc::UnboundedSender;
use unicode_width::UnicodeWidthStr;
use yazi_config::{popup::Position, INPUT};
use yazi_plugin::CLIPBOARD;
use yazi_shared::{render, InputError};

use super::{mode::InputMode, op::InputOp, InputSnap, InputSnaps};

#[derive(Default)]
pub struct Input {
	pub(super) snaps: InputSnaps,
	pub ticket:       usize,
	pub visible:      bool,

	pub title:    String,
	pub position: Position,

	// Typing
	pub(super) callback:   Option<UnboundedSender<Result<String, InputError>>>,
	pub(super) realtime:   bool,
	pub(super) completion: bool,

	// Shell
	pub highlight: bool,
}

impl Input {
	#[inline]
	pub(super) fn limit(&self) -> usize {
		self.position.offset.width.saturating_sub(INPUT.border()) as usize
	}

	pub fn type_str(&mut self, s: &str) {
		let snap = self.snaps.current_mut();
		if snap.cursor < 1 {
			snap.value.insert_str(0, s);
		} else {
			snap.value.insert_str(snap.idx(snap.cursor).unwrap(), s);
		}

		self.move_(s.chars().count() as isize);
		self.flush_value();
		render!();
	}

	pub(super) fn handle_op(&mut self, cursor: usize, include: bool) -> bool {
		let old = self.snap().clone();
		let snap = self.snaps.current_mut();

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
			self.snaps.tag(self.limit()).then(|| self.flush_value());
		}
		true
	}

	pub(super) fn flush_value(&mut self) {
		let Some(tx) = &self.callback else { return };
		self.ticket = self.ticket.wrapping_add(1);

		if self.realtime {
			let value = self.snap().value.clone();
			tx.send(Err(InputError::Typed(value))).ok();
		}

		if self.completion {
			let before = self.partition()[0].to_owned();
			tx.send(Err(InputError::Completed(before, self.ticket))).ok();
		}
	}
}

impl Input {
	#[inline]
	pub fn value(&self) -> &str { self.snap().slice(self.snap().window(self.limit())) }

	#[inline]
	pub fn mode(&self) -> InputMode { self.snap().mode }

	#[inline]
	pub fn cursor(&self) -> u16 {
		let snap = self.snap();
		snap.slice(snap.offset..snap.cursor).width() as u16
	}

	pub fn selected(&self) -> Option<Range<u16>> {
		let snap = self.snap();
		let start = snap.op.start()?;

		let (start, end) =
			if start < snap.cursor { (start, snap.cursor) } else { (snap.cursor + 1, start + 1) };

		let win = snap.window(self.limit());
		let Range { start, end } = start.max(win.start)..end.min(win.end);

		let s = snap.slice(snap.offset..start).width() as u16;
		Some(s..s + snap.slice(start..end).width() as u16)
	}

	#[inline]
	pub fn partition(&self) -> [&str; 2] {
		let snap = self.snap();
		let idx = snap.idx(snap.cursor).unwrap();
		[&snap.value[..idx], &snap.value[idx..]]
	}

	#[inline]
	pub(super) fn snap(&self) -> &InputSnap { self.snaps.current() }

	#[inline]
	pub(super) fn snap_mut(&mut self) -> &mut InputSnap { self.snaps.current_mut() }
}
