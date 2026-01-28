use std::{io, ops::Deref};

use anyhow::Result;
use crossterm::{event::{DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags}, execute, queue, style::Print, terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode}};
use ratatui::{CompletedFrame, Frame, Terminal, backend::CrosstermBackend, buffer::Buffer, layout::Rect};
use yazi_emulator::{Emulator, Mux, TMUX};
use yazi_shared::SyncCell;
use yazi_shim::crossterm::{If, RestoreBackground, RestoreCursor, SetBackground};
use yazi_tty::{TTY, TtyWriter};

use crate::{TermOption, TermState};

static STATE: SyncCell<TermState> = SyncCell::new(TermState::default());

pub struct Term {
	inner:       Terminal<CrosstermBackend<TtyWriter<'static>>>,
	last_area:   Rect,
	last_buffer: Buffer,
}

impl Term {
	pub fn start() -> Result<Self> {
		let opt = TermOption::default();
		let mut term = Self {
			inner:       Terminal::new(CrosstermBackend::new(TTY.writer()))?,
			last_area:   Default::default(),
			last_buffer: Default::default(),
		};

		enable_raw_mode()?;
		static FIRST: SyncCell<bool> = SyncCell::new(false);
		if FIRST.replace(true) && yazi_emulator::TMUX.get() {
			yazi_emulator::Mux::tmux_passthrough();
		}

		execute!(
			TTY.writer(),
			If(!TMUX.get(), EnterAlternateScreen),
			Print("\x1bP$q q\x1b\\"), // Request cursor shape (DECRQSS query for DECSCUSR)
			Print("\x1b[?12$p"),      // Request cursor blink status (DECRQM query for DECSET 12)
			Print("\x1b[?u"),         // Request keyboard enhancement flags (CSI u)
			Print("\x1b[0c"),         // Request device attributes
			If(TMUX.get(), EnterAlternateScreen),
			SetBackground(&opt.bg), // Set app background
			EnableBracketedPaste,
			EnableFocusChange,
			If(opt.mouse, EnableMouseCapture),
		)?;

		let resp = Emulator::read_until_da1();
		Mux::tmux_drain()?;

		STATE.set(TermState::new(&resp, &opt));
		if STATE.get().csi_u {
			_ = queue!(
				TTY.writer(),
				PushKeyboardEnhancementFlags(
					KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
						| KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS,
				)
			);
		}

		if let Some(s) = opt.title {
			queue!(TTY.writer(), SetTitle(s)).ok();
		}

		term.inner.hide_cursor()?;
		term.inner.clear()?;
		term.inner.flush()?;
		Ok(term)
	}

	fn stop(&mut self) -> Result<()> {
		let state = STATE.get();

		execute!(
			TTY.writer(),
			If(state.mouse, DisableMouseCapture),
			If(state.bg, RestoreBackground),
			If(state.csi_u, PopKeyboardEnhancementFlags),
			RestoreCursor { shape: state.cursor_shape, blink: state.cursor_blink },
			If(state.title, SetTitle("")),
			DisableFocusChange,
			DisableBracketedPaste,
			LeaveAlternateScreen,
		)?;

		self.inner.show_cursor()?;
		Ok(disable_raw_mode()?)
	}

	pub fn goodbye(f: impl FnOnce() -> i32) -> ! {
		let state = STATE.get();

		execute!(
			TTY.writer(),
			If(state.mouse, DisableMouseCapture),
			If(state.bg, RestoreBackground),
			If(state.csi_u, PopKeyboardEnhancementFlags),
			RestoreCursor { shape: state.cursor_shape, blink: state.cursor_blink },
			If(state.title, SetTitle("")),
			DisableFocusChange,
			DisableBracketedPaste,
			LeaveAlternateScreen,
			crossterm::cursor::Show
		)
		.ok();

		disable_raw_mode().ok();

		std::process::exit(f());
	}

	pub fn draw(&mut self, f: impl FnOnce(&mut Frame)) -> io::Result<CompletedFrame<'_>> {
		let last = self.inner.draw(f)?;

		self.last_area = last.area;
		self.last_buffer = last.buffer.clone();
		Ok(last)
	}

	pub fn draw_partial(&mut self, f: impl FnOnce(&mut Frame)) -> io::Result<CompletedFrame<'_>> {
		self.inner.draw(|frame| {
			let buffer = frame.buffer_mut();
			for y in self.last_area.top()..self.last_area.bottom() {
				for x in self.last_area.left()..self.last_area.right() {
					let mut cell = self.last_buffer[(x, y)].clone();
					cell.skip = false;
					buffer[(x, y)] = cell;
				}
			}

			f(frame);
		})
	}

	pub fn can_partial(&mut self) -> bool {
		self.inner.autoresize().is_ok() && self.last_area == self.inner.get_frame().area()
	}
}

impl Drop for Term {
	fn drop(&mut self) { self.stop().ok(); }
}

impl Deref for Term {
	type Target = Terminal<CrosstermBackend<TtyWriter<'static>>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}
