use std::{io::{self, BufWriter, Stderr, stderr}, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, AtomicU8, Ordering}};

use anyhow::Result;
use crossterm::{event::{DisableBracketedPaste, EnableBracketedPaste, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags}, execute, queue, style::Print, terminal::{LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode}};
use cursor::RestoreCursor;
use ratatui::{CompletedFrame, Frame, Terminal, backend::CrosstermBackend, buffer::Buffer, layout::Rect};
use yazi_adapter::{Emulator, Mux};
use yazi_config::{INPUT, MGR};

static CSI_U: AtomicBool = AtomicBool::new(false);
static BLINK: AtomicBool = AtomicBool::new(false);
static SHAPE: AtomicU8 = AtomicU8::new(0);

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
		if yazi_adapter::TMUX.get() {
			yazi_adapter::Mux::tmux_passthrough();
		}

		execute!(
			BufWriter::new(stderr()),
			screen::SetScreen(true),
			Print(Mux::csi("\x1bP$q q\x1b\\")), // Request cursor shape (DECRQSS query for DECSCUSR)
			Print(Mux::csi("\x1b[?12$p")),      // Request cursor blink status (DECSET)
			Print("\x1b[?u"),                   // Request keyboard enhancement flags (CSI u)
			Print(Mux::csi("\x1b[0c")),         // Request device attributes
			screen::SetScreen(false),
			EnableBracketedPaste,
			mouse::SetMouse(true),
		)?;

		let resp = Emulator::read_until_da1();
		Mux::tmux_drain()?;

		CSI_U.store(resp.contains("\x1b[?0u"), Ordering::Relaxed);
		BLINK.store(resp.contains("\x1b[?12;1$y"), Ordering::Relaxed);
		SHAPE.store(
			resp
				.split_once("\x1bP1$r")
				.and_then(|(_, s)| s.bytes().next())
				.filter(|&b| matches!(b, b'0'..=b'6'))
				.map_or(u8::MAX, |b| b - b'0'),
			Ordering::Relaxed,
		);

		if CSI_U.load(Ordering::Relaxed) {
			queue!(
				stderr(),
				PushKeyboardEnhancementFlags(
					KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
						| KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
				)
			)?;
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
			RestoreCursor,
			DisableBracketedPaste,
			LeaveAlternateScreen,
		)?;

		self.show_cursor()?;
		Ok(disable_raw_mode()?)
	}

	pub(super) fn goodbye(f: impl FnOnce() -> bool) -> ! {
		if CSI_U.swap(false, Ordering::Relaxed) {
			execute!(stderr(), PopKeyboardEnhancementFlags).ok();
		}

		if !MGR.title_format.is_empty() {
			execute!(stderr(), SetTitle("")).ok();
		}

		execute!(
			stderr(),
			mouse::SetMouse(false),
			RestoreCursor,
			SetTitle(""),
			DisableBracketedPaste,
			LeaveAlternateScreen,
			crossterm::cursor::Show
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

	#[inline]
	pub(super) fn set_cursor_underscore() -> Result<()> {
		use crossterm::cursor::SetCursorStyle;
		Ok(if INPUT.cursor_blink {
			queue!(stderr(), SetCursorStyle::BlinkingUnderScore)?
		} else {
			queue!(stderr(), SetCursorStyle::SteadyUnderScore)?
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
	use yazi_config::MGR;

	pub struct SetMouse(pub bool);

	impl crossterm::Command for SetMouse {
		fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
			if MGR.mouse_events.is_empty() {
				Ok(())
			} else if self.0 {
				EnableMouseCapture.write_ansi(f)
			} else {
				DisableMouseCapture.write_ansi(f)
			}
		}

		#[cfg(windows)]
		fn execute_winapi(&self) -> std::io::Result<()> {
			if MGR.mouse_events.is_empty() {
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

// --- Cursor shape
mod cursor {
	use std::sync::atomic::Ordering;

	use crossterm::cursor::SetCursorStyle;

	use super::{BLINK, SHAPE};

	pub struct RestoreCursor;

	impl crossterm::Command for RestoreCursor {
		fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
			let (shape, shape_blink) = match SHAPE.load(Ordering::Relaxed) {
				u8::MAX => (0, false),
				n => (n.max(1).div_ceil(2), n.max(1) & 1 == 1),
			};

			let blink = BLINK.load(Ordering::Relaxed) ^ shape_blink;
			Ok(match shape {
				2 if blink => SetCursorStyle::BlinkingUnderScore.write_ansi(f)?,
				2 if !blink => SetCursorStyle::SteadyUnderScore.write_ansi(f)?,
				3 if blink => SetCursorStyle::BlinkingBar.write_ansi(f)?,
				3 if !blink => SetCursorStyle::SteadyBar.write_ansi(f)?,
				_ if blink => SetCursorStyle::DefaultUserShape.write_ansi(f)?,
				_ if !blink => SetCursorStyle::SteadyBlock.write_ansi(f)?,
				_ => unreachable!(),
			})
		}

		#[cfg(windows)]
		fn execute_winapi(&self) -> std::io::Result<()> { Ok(()) }
	}
}

mod screen {
	use crossterm::terminal::EnterAlternateScreen;
	use yazi_adapter::TMUX;

	pub struct SetScreen(pub bool);

	impl crossterm::Command for SetScreen {
		fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
			if self.0 == TMUX.get() { Ok(()) } else { EnterAlternateScreen.write_ansi(f) }
		}

		#[cfg(windows)]
		fn execute_winapi(&self) -> std::io::Result<()> {
			if self.0 == TMUX.get() { Ok(()) } else { EnterAlternateScreen.execute_winapi() }
		}

		#[cfg(windows)]
		fn is_ansi_code_supported(&self) -> bool { EnterAlternateScreen.is_ansi_code_supported() }
	}
}
