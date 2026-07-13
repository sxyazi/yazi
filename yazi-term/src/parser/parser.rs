use std::{collections::VecDeque, mem, num::NonZeroU8, str};

use yazi_shim::utf8_char_width;

use super::state::State;
use crate::event::{ClipboardEvent, DndEvent, Event, KeyCode, KeyEvent, Modifiers};

#[derive(Debug)]
pub struct Parser {
	pub(super) state:  State,
	pub(super) seq:    Vec<u8>,
	pub(crate) events: VecDeque<Event>,
}

impl Default for Parser {
	fn default() -> Self {
		Self {
			state:  State::Ground,
			seq:    Vec::with_capacity(64),
			events: VecDeque::with_capacity(32),
		}
	}
}

impl Parser {
	pub fn parse(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.step(b);
		}
	}

	fn step(&mut self, b: u8) {
		match &self.state {
			State::Ground => self.on_ground(b),
			State::Esc => self.on_esc(b),
			State::EscO => self.on_esco(b),
			State::Csi => self.on_csi(b),
			State::NormalMouse => self.on_normal_mouse(b),
			State::BracketedPaste => self.on_bracketed_paste(b),
			State::Osc | State::OscSt => self.on_osc(b),
			State::Osc72(_) => self.on_osc72(b),
			State::Osc5522(_) => self.on_osc5522(b),
			State::Dcs | State::DcsSt => self.on_dcs(b),
			State::Utf8(n) => self.on_utf8(b, *n),
			State::AltUtf8(n) => self.on_alt_utf8(b, *n),
		}
	}

	/// Resolve any pending ambiguous state.
	///
	/// Call this when no more input bytes are immediately available. If the
	/// parser is waiting in the [`State::Esc`] state (a lone `\x1B` has
	/// been seen but no follow-up bytes arrived), this emits a bare
	/// [`KeyCode::Escape`] event and resets to [`State::Ground`].
	pub fn flush(&mut self) {
		match &self.state {
			State::Esc => self.emit_key(KeyCode::Escape),
			State::Osc72(s) if s.has_more => return,
			State::Osc5522(s) if s.has_more => return,
			_ => {}
		}

		self.reset();
	}

	fn reset(&mut self) {
		self.state = State::Ground;
		self.seq.clear();
	}

	pub(crate) fn emit(&mut self, event: impl Into<Event>) { self.events.push_back(event.into()); }

	fn emit_key(&mut self, event: impl Into<KeyEvent>) { self.emit(Event::Key(event.into())); }

	pub fn pop(&mut self) -> Option<Event> { self.events.pop_front() }

	fn on_ground(&mut self, b: u8) {
		match b {
			b'\x1B' => {
				self.seq.clear();
				self.seq.push(b'\x1B');
				self.state = State::Esc;
			}
			_ if let Some(key) = Self::parse_ground_key(b) => {
				self.emit_key(key);
			}
			_ if let w @ 2..=4 = utf8_char_width(b) => {
				self.seq.clear();
				self.seq.push(b);
				self.state = State::Utf8(NonZeroU8::new(w - 1).unwrap());
			}
			_ => {}
		}
	}

	fn on_esc(&mut self, b: u8) {
		self.seq.push(b);
		match b {
			b'[' => self.state = State::Csi,
			b']' => self.state = State::Osc,
			b'P' => self.state = State::Dcs,
			b'O' => self.state = State::EscO,
			// ESC ESC: emit Escape for the first byte, the second is consumed.
			b'\x1B' => {
				self.emit_key(KeyCode::Escape);
				self.reset();
			}
			// Alt + <key>
			_ if let Some(mut key) = Self::parse_ground_key(b) => {
				key.modifiers |= Modifiers::ALT;
				self.emit_key(key);
				self.reset();
			}
			// seq = [ESC, b], collect w-1 more continuation bytes.
			_ if let w @ 2..=4 = utf8_char_width(b) => {
				self.state = State::AltUtf8(NonZeroU8::new(w - 1).unwrap());
			}
			// invalid byte — discard
			_ => self.reset(),
		}
	}

	fn on_esco(&mut self, b: u8) {
		self.seq.push(b);
		let event = match b {
			b'A' => Event::Key(KeyCode::Up.into()),
			b'B' => Event::Key(KeyCode::Down.into()),
			b'C' => Event::Key(KeyCode::Right.into()),
			b'D' => Event::Key(KeyCode::Left.into()),
			b'F' => Event::Key(KeyCode::End.into()),
			b'H' => Event::Key(KeyCode::Home.into()),
			v @ b'P'..=b'S' => Event::Key(KeyCode::Fn(1 + v - b'P').into()),
			_ => {
				self.reset();
				return;
			}
		};
		self.emit(event);
		self.reset();
	}

	fn on_csi(&mut self, b: u8) {
		self.seq.push(b);

		// Bytes below 0x40 are parameter or intermediate bytes; keep accumulating.
		if !(0x40..=0x7e).contains(&b) {
			return;
		}

		// The Linux console sends `\x1B[[x` for F1–F5. The inner `[` (0x5B) is
		// technically in the final-byte range but must not trigger early dispatch.
		if b == b'[' && self.seq.len() == 3 {
			return;
		}

		// `\x1B[M` (no params) = X10 normal-mouse encoding; 3 raw bytes follow.
		if b == b'M' && self.seq.len() == 3 {
			self.state = State::NormalMouse;
			return;
		}

		// `\x1B[200~` = start of bracketed-paste mode.
		if self.seq == b"\x1B[200~" {
			self.state = State::BracketedPaste;
			return;
		}

		if let Ok(e) = self.parse_csi() {
			self.emit(e);
		}
		self.reset();
	}

	fn on_normal_mouse(&mut self, b: u8) {
		self.seq.push(b);

		// Need `\x1B[M` + 3 raw bytes = 6 bytes total.
		if self.seq.len() < 6 {
			return;
		}

		if let Ok(e) = self.parse_csi_normal_mouse() {
			self.emit(e);
		}
		self.reset();
	}

	fn on_bracketed_paste(&mut self, b: u8) {
		self.seq.push(b);

		if self.seq.ends_with(b"\x1b[201~") {
			// seq = b"\x1B[200~" + paste_content + b"\x1B[201~"
			let paste = String::from_utf8_lossy(&self.seq[6..self.seq.len() - 6]).into_owned();
			self.emit(Event::Paste(paste));
			self.reset();
		}
	}

	fn on_osc(&mut self, b: u8) {
		self.seq.push(b);

		match (&self.state, b) {
			(State::Osc, b'\x07') => self.reset(), // BEL — OSC complete (discard)
			(State::Osc, _) if self.seq.starts_with(b"\x1b]72;") => {
				self.state = State::Osc72(Default::default());
			}
			(State::Osc, _) if self.seq.starts_with(b"\x1b]5522;") => {
				self.state = State::Osc5522(Default::default());
			}
			(State::Osc, b'\x1B') => self.state = State::OscSt,
			(State::Osc, _) => {}                         // keep accumulating
			(State::OscSt, b'\\') => self.reset(),        // ST (`\x1B\\`) — OSC complete (discard)
			(State::OscSt, b'\x1B') => {}                 // another ESC — stay in OscSt
			(State::OscSt, _) => self.state = State::Osc, // not ST — resume OSC
			_ => unreachable!(),
		}
	}

	fn on_osc72(&mut self, b: u8) {
		self.seq.push(b);

		if !self.seq.ends_with(b"\x1b\\") {
			return;
		} else if self.parse_osc72().is_err() {
			return self.reset();
		}

		match mem::take(&mut self.state) {
			State::Osc72(s) if s.has_more => {
				self.seq.clear();
				self.state = State::Osc72(s);
			}
			State::Osc72(s) => {
				if let Some(e) = DndEvent::from_state(s) {
					self.emit(Event::Dnd(e));
				}
				self.reset();
			}
			_ => unreachable!(),
		}
	}

	fn on_osc5522(&mut self, b: u8) {
		self.seq.push(b);

		if !self.seq.ends_with(b"\x1b\\") {
			return;
		} else if self.parse_osc5522().is_err() {
			return self.reset();
		}

		match mem::take(&mut self.state) {
			State::Osc5522(s) if s.has_more => {
				self.seq.clear();
				self.state = State::Osc5522(s);
			}
			State::Osc5522(s) => {
				if let Some(e) = ClipboardEvent::from_state(s) {
					self.emit(Event::Clipboard(e));
				}
				self.reset();
			}
			_ => unreachable!(),
		}
	}

	fn on_dcs(&mut self, b: u8) {
		self.seq.push(b);

		match (&self.state, b) {
			(State::Dcs, b'\x1B') => self.state = State::DcsSt,
			(State::Dcs, _) => {}
			(State::DcsSt, b'\\') => self.reset(), // ST — DCS complete (discard)
			(State::DcsSt, b'\x1B') => {}          // another ESC — stay in DcsSt
			(State::DcsSt, _) => self.state = State::Dcs, // not ST — resume DCS
			_ => unreachable!(),
		}
	}

	fn on_utf8(&mut self, b: u8, remaining: NonZeroU8) {
		if b & 0xc0 != 0x80 {
			self.reset();
			self.on_ground(b);
			return; // Not a continuation byte — abandon the sequence and retry from ground.
		}

		self.seq.push(b);
		if remaining.get() != 1 {
			return self.state = State::Utf8(NonZeroU8::new(remaining.get() - 1).unwrap());
		}

		if let Ok(s) = str::from_utf8(&self.seq)
			&& let Some(c) = s.chars().next()
		{
			self.emit_key(KeyEvent::new(KeyCode::Char(c), Modifiers::for_char(c)));
		}
		self.reset();
	}

	fn on_alt_utf8(&mut self, b: u8, remaining: NonZeroU8) {
		if b & 0xc0 != 0x80 {
			self.reset();
			self.on_ground(b);
			return; // Not a continuation byte — abandon Alt+UTF-8 and retry from ground.
		}

		self.seq.push(b);
		if remaining.get() != 1 {
			return self.state = State::AltUtf8(NonZeroU8::new(remaining.get() - 1).unwrap());
		}

		// seq = [ESC, …utf8_bytes…]
		if let Ok(s) = str::from_utf8(&self.seq[1..])
			&& let Some(c) = s.chars().next()
		{
			self.emit_key(KeyEvent::new(KeyCode::Char(c), Modifiers::ALT | Modifiers::for_char(c)));
		}
		self.reset();
	}
}
