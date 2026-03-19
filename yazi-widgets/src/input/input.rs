use std::{borrow::Cow, ops::Range};

use anyhow::Result;
use crossterm::cursor::SetCursorStyle;
use tokio::sync::mpsc;
use yazi_config::YAZI;
use yazi_macro::act;
use yazi_shared::Ids;

use super::{InputSnap, InputSnaps, mode::InputMode, op::InputOp};
use crate::{CLIPBOARD, input::{InputEvent, InputOpt, SEPARATOR}};

#[derive(Default)]
pub struct Input {
	pub snaps:      InputSnaps,
	pub limit:      usize,
	pub obscure:    bool,
	pub realtime:   bool,
	pub completion: bool,

	pub tx:     Option<mpsc::UnboundedSender<InputEvent>>,
	pub ticket: Ids,
}

impl Input {
	pub fn new(opt: InputOpt) -> Result<Self> {
		let limit = opt.cfg.position.offset.width.saturating_sub(YAZI.input.border()) as usize;
		let mut input = Self {
			snaps: InputSnaps::new(opt.cfg.value, opt.cfg.obscure, limit),
			limit,
			obscure: opt.cfg.obscure,
			realtime: opt.cfg.realtime,
			completion: opt.cfg.completion,

			tx: Some(opt.tx),
			..Default::default()
		};

		if let Some(cursor) = opt.cfg.cursor {
			input.snap_mut().cursor = cursor;
			act!(r#move, input)?;
		}

		Ok(input)
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
			self.snaps.tag(self.limit).then(|| self.flush_type());
		}
		true
	}

	pub(super) fn flush_type(&mut self) {
		self.ticket.next();
		if let Some(tx) = self.tx.as_ref().filter(|_| self.realtime) {
			tx.send(InputEvent::Type(self.value().to_owned())).ok();
		}

		self.flush_trigger(true);
	}

	pub(super) fn flush_trigger(&self, force: bool) {
		if let Some(tx) = self.tx.as_ref().filter(|_| self.completion) {
			tx.send(InputEvent::Trigger(
				self.partition().0.to_owned(),
				Some(self.ticket.current()).filter(|_| !force),
			))
			.ok();
		}
	}
}

impl Input {
	pub fn value(&self) -> &str { &self.snap().value }

	pub fn display(&self) -> Cow<'_, str> {
		if self.obscure {
			"•".repeat(self.snap().window(self.limit).len()).into()
		} else {
			self.snap().slice(self.snap().window(self.limit)).into()
		}
	}

	pub fn mode(&self) -> InputMode { self.snap().mode }

	pub fn cursor(&self) -> u16 { self.snap().width(self.snap().offset..self.snap().cursor) }

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

		let s = snap.width(snap.offset..start);
		Some(s..s + snap.width(start..end))
	}

	pub fn partition(&self) -> (&str, &str) {
		let snap = self.snap();
		let idx = snap.idx(snap.cursor).unwrap();

		if let Some(sep) = snap.value[idx..].find(SEPARATOR).map(|i| idx + i) {
			(&snap.value[..sep], &snap.value[sep + 1..])
		} else {
			(&snap.value, "")
		}
	}

	pub fn snap(&self) -> &InputSnap { self.snaps.current() }

	pub fn snap_mut(&mut self) -> &mut InputSnap { self.snaps.current_mut() }
}
