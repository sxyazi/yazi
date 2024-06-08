use std::{io::{self, stderr, BufWriter, Stderr}, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

use anyhow::Result;
use crossterm::{cursor::{RestorePosition, SavePosition}, event::{DisableBracketedPaste, EnableBracketedPaste, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags}, execute, queue, style::Print, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle}};
use ratatui::{backend::CrosstermBackend, buffer::Buffer, layout::Rect, CompletedFrame, Frame, Terminal};
use yazi_adaptor::Emulator;
use yazi_config::INPUT;

static CSI_U: AtomicBool = AtomicBool::new(false);

pub(super) struct Term {
	inner:       Terminal<CrosstermBackend<BufWriter<Stderr>>>,
	last_area:   Rect,
	last_buffer: Buffer,
}

impl Term {
	pub(super) fn start() -> Result<Self> {
		let mut term = Self {
			inner:       Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?,
			last_area:   Default::default(),
			last_buffer: Default::default(),
		};

		enable_raw_mode()?;
		execute!(
			BufWriter::new(stderr()),
			EnterAlternateScreen,
			EnableBracketedPaste,
			mouse::SetMouse(true),
			SavePosition,
			Print("\x1b[?u\x1b[c"),
			RestorePosition
		)?;

		let resp = futures::executor::block_on(Emulator::read_until_da1());
		if resp.is_ok_and(|s| s.contains("\x1b[?0u")) {
			queue!(
				stderr(),
				PushKeyboardEnhancementFlags(
					KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
						| KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
				)
			)?;
			CSI_U.store(true, Ordering::Relaxed);
		}

		term.hide_cursor()?;
		term.clear()?;
		term.flush()?;
		Ok(term)
	}

	fn stop(&mut self) -> Result<()> {
		if CSI_U.swap(false, Ordering::Relaxed) {
			execute!(stderr(), PopKeyboardEnhancementFlags)?;
		}

		execute!(
			stderr(),
			mouse::SetMouse(false),
			DisableBracketedPaste,
			LeaveAlternateScreen,
			crossterm::cursor::SetCursorStyle::DefaultUserShape
		)?;

		self.show_cursor()?;
		Ok(disable_raw_mode()?)
	}

	pub(super) fn goodbye(f: impl FnOnce() -> bool) -> ! {
		if CSI_U.swap(false, Ordering::Relaxed) {
			execute!(stderr(), PopKeyboardEnhancementFlags).ok();
		}

		execute!(
			stderr(),
			SetTitle(""),
			mouse::SetMouse(false),
			DisableBracketedPaste,
			LeaveAlternateScreen,
			crossterm::cursor::SetCursorStyle::DefaultUserShape,
			crossterm::cursor::Show,
		)
		.ok();

		disable_raw_mode().ok();

		std::process::exit(f() as i32);
	}

	pub(super) fn draw(&mut self, f: impl FnOnce(&mut Frame)) -> io::Result<CompletedFrame> {
		let last = self.inner.draw(f)?;

		self.last_area = last.area;
		self.last_buffer = last.buffer.clone();
		Ok(last)
	}

	pub(super) fn draw_partial(&mut self, f: impl FnOnce(&mut Frame)) -> io::Result<CompletedFrame> {
		self.inner.draw(|frame| {
			let buffer = frame.buffer_mut();
			for y in self.last_area.top()..self.last_area.bottom() {
				for x in self.last_area.left()..self.last_area.right() {
					let mut cell = self.last_buffer.get(x, y).clone();
					cell.skip = false;
					*buffer.get_mut(x, y) = cell;
				}
			}

			f(frame);
		})
	}

	#[inline]
	pub(super) fn can_partial(&mut self) -> bool {
		self.inner.autoresize().is_ok() && self.last_area == self.inner.get_frame().size()
	}

	#[inline]
	pub(super) fn set_cursor_block() -> Result<()> {
		use crossterm::cursor::SetCursorStyle;
		Ok(if INPUT.cursor_blink {
			queue!(stderr(), SetCursorStyle::BlinkingBlock)?
		} else {
			queue!(stderr(), SetCursorStyle::SteadyBlock)?
		})
	}

	#[inline]
	pub(super) fn set_cursor_bar() -> Result<()> {
		use crossterm::cursor::SetCursorStyle;
		Ok(if INPUT.cursor_blink {
			queue!(stderr(), SetCursorStyle::BlinkingBar)?
		} else {
			queue!(stderr(), SetCursorStyle::SteadyBar)?
		})
	}
}

impl Drop for Term {
	fn drop(&mut self) { self.stop().ok(); }
}

impl Deref for Term {
	type Target = Terminal<CrosstermBackend<BufWriter<Stderr>>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Term {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

// --- Mouse support
mod mouse {
	use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
	use yazi_config::MANAGER;

	pub struct SetMouse(pub bool);

	impl crossterm::Command for SetMouse {
		fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
			if MANAGER.mouse_events.is_empty() {
				Ok(())
			} else if self.0 {
				EnableMouseCapture.write_ansi(f)
			} else {
				DisableMouseCapture.write_ansi(f)
			}
		}

		#[cfg(windows)]
		fn execute_winapi(&self) -> std::io::Result<()> {
			if MANAGER.mouse_events.is_empty() {
				Ok(())
			} else if self.0 {
				EnableMouseCapture.execute_winapi()
			} else {
				DisableMouseCapture.execute_winapi()
			}
		}

		#[cfg(windows)]
		fn is_ansi_code_supported(&self) -> bool {
			if self.0 {
				EnableMouseCapture.is_ansi_code_supported()
			} else {
				DisableMouseCapture.is_ansi_code_supported()
			}
		}
	}
}
