use std::{io, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

use anyhow::Result;
use crossterm::{Command, event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags}, execute, queue, style::Print, terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode}};
use ratatui::{CompletedFrame, Frame, Terminal, backend::CrosstermBackend, buffer::Buffer, layout::Rect};
use yazi_adapter::{Emulator, Mux, TMUX};
use yazi_config::YAZI;
use yazi_shared::SyncCell;
use yazi_term::tty::{TTY, TtyWriter};

static CSI_U: AtomicBool = AtomicBool::new(false);

pub(super) struct Term {
	inner:       Terminal<CrosstermBackend<TtyWriter<'static>>>,
	last_area:   Rect,
	last_buffer: Buffer,
}

impl Term {
	pub(super) fn start() -> Result<Self> {
		static SKIP: SyncCell<bool> = SyncCell::new(false);
		let mut term = Self {
			inner:       Terminal::new(CrosstermBackend::new(TTY.writer()))?,
			last_area:   Default::default(),
			last_buffer: Default::default(),
		};

		enable_raw_mode()?;
		if SKIP.replace(true) && yazi_adapter::TMUX.get() {
			yazi_adapter::Mux::tmux_passthrough();
		}

		execute!(
			TTY.writer(),
			yazi_term::If(!TMUX.get(), EnterAlternateScreen),
			Print("\x1bP$q q\x1b\\"), // Request cursor shape (DECRQSS query for DECSCUSR)
			Print(Mux::csi("\x1b[?12$p")), // Request cursor blink status (DECSET)
			Print("\x1b[?u"),         // Request keyboard enhancement flags (CSI u)
			Print(Mux::csi("\x1b[0c")), // Request device attributes
			yazi_term::If(TMUX.get(), EnterAlternateScreen),
			EnableBracketedPaste,
			yazi_term::If(!YAZI.mgr.mouse_events.is_empty(), EnableMouseCapture),
		)?;

		let resp = Emulator::read_until_da1();
		Mux::tmux_drain()?;
		yazi_term::RestoreCursor::store(&resp);

		CSI_U.store(resp.contains("\x1b[?0u"), Ordering::Relaxed);
		if CSI_U.load(Ordering::Relaxed) {
			PushKeyboardEnhancementFlags(
				KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
					| KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS,
			)
			.write_ansi(&mut TTY.writer())?;
		}

		if let Some(s) = YAZI.mgr.title() {
			queue!(TTY.writer(), SetTitle(s)).ok();
		}

		term.hide_cursor()?;
		term.clear()?;
		term.flush()?;
		Ok(term)
	}

	fn stop(&mut self) -> Result<()> {
		if CSI_U.swap(false, Ordering::Relaxed) {
			PopKeyboardEnhancementFlags.write_ansi(&mut TTY.writer())?;
		}

		if !YAZI.mgr.title_format.is_empty() {
			queue!(TTY.writer(), SetTitle("")).ok();
		}

		execute!(
			TTY.writer(),
			yazi_term::If(!YAZI.mgr.mouse_events.is_empty(), DisableMouseCapture),
			yazi_term::RestoreCursor,
			DisableBracketedPaste,
			LeaveAlternateScreen,
		)?;

		self.show_cursor()?;
		Ok(disable_raw_mode()?)
	}

	pub(super) fn goodbye(f: impl FnOnce() -> i32) -> ! {
		if CSI_U.swap(false, Ordering::Relaxed) {
			PopKeyboardEnhancementFlags.write_ansi(&mut TTY.writer()).ok();
		}

		if !YAZI.mgr.title_format.is_empty() {
			queue!(TTY.writer(), SetTitle("")).ok();
		}

		execute!(
			TTY.writer(),
			yazi_term::If(!YAZI.mgr.mouse_events.is_empty(), DisableMouseCapture),
			yazi_term::RestoreCursor,
			DisableBracketedPaste,
			LeaveAlternateScreen,
			crossterm::cursor::Show
		)
		.ok();

		disable_raw_mode().ok();

		std::process::exit(f());
	}

	pub(super) fn draw(&mut self, f: impl FnOnce(&mut Frame)) -> io::Result<CompletedFrame<'_>> {
		let last = self.inner.draw(f)?;

		self.last_area = last.area;
		self.last_buffer = last.buffer.clone();
		Ok(last)
	}

	pub(super) fn draw_partial(
		&mut self,
		f: impl FnOnce(&mut Frame),
	) -> io::Result<CompletedFrame<'_>> {
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

	#[inline]
	pub(super) fn can_partial(&mut self) -> bool {
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

impl DerefMut for Term {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}
