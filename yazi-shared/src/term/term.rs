use std::{io::{self, stderr, BufWriter, Stderr, Write}, mem, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

use anyhow::Result;
use crossterm::{cursor::{RestorePosition, SavePosition}, event::{DisableBracketedPaste, DisableFocusChange, EnableBracketedPaste, EnableFocusChange, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags}, execute, queue, style::Print, terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, WindowSize}};
use ratatui::{backend::CrosstermBackend, buffer::Buffer, layout::Rect, CompletedFrame, Frame, Terminal};

static CSI_U: AtomicBool = AtomicBool::new(false);

pub struct Term {
	inner:       Terminal<CrosstermBackend<BufWriter<Stderr>>>,
	last_area:   Rect,
	last_buffer: Buffer,
}

impl Term {
	pub fn start() -> Result<Self> {
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
			EnableFocusChange,
			SavePosition,
			Print("\x1b[?u\x1b[c"),
			RestorePosition
		)?;

		let resp = futures::executor::block_on(Self::read_until_da1()).unwrap_or_default();
		if resp.contains("\x1b[?0u") {
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
			DisableFocusChange,
			DisableBracketedPaste,
			LeaveAlternateScreen,
			crossterm::cursor::SetCursorStyle::DefaultUserShape
		)?;

		self.show_cursor()?;
		Ok(disable_raw_mode()?)
	}

	pub fn goodbye(f: impl FnOnce() -> bool) -> ! {
		if CSI_U.swap(false, Ordering::Relaxed) {
			execute!(stderr(), PopKeyboardEnhancementFlags).ok();
		}

		execute!(
			stderr(),
			DisableFocusChange,
			DisableBracketedPaste,
			LeaveAlternateScreen,
			crossterm::cursor::SetCursorStyle::DefaultUserShape,
			crossterm::cursor::Show,
		)
		.ok();

		disable_raw_mode().ok();

		std::process::exit(f() as i32);
	}

	pub fn draw(&mut self, f: impl FnOnce(&mut Frame)) -> io::Result<CompletedFrame> {
		let last = self.inner.draw(f)?;

		self.last_area = last.area;
		self.last_buffer = last.buffer.clone();
		Ok(last)
	}

	pub fn draw_partial(&mut self, f: impl FnOnce(&mut Frame)) -> io::Result<CompletedFrame> {
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
	pub fn can_partial(&mut self) -> bool { self.last_area == self.inner.get_frame().size() }

	pub fn size() -> WindowSize {
		let mut size = WindowSize { rows: 0, columns: 0, width: 0, height: 0 };
		if let Ok(s) = crossterm::terminal::window_size() {
			_ = mem::replace(&mut size, s);
		}

		if size.rows == 0 || size.columns == 0 {
			if let Ok(s) = crossterm::terminal::size() {
				size.columns = s.0;
				size.rows = s.1;
			}
		}

		// TODO: Use `CSI 14 t` to get the actual size of the terminal
		// if size.width == 0 || size.height == 0 {}

		size
	}

	#[inline]
	pub fn ratio() -> Option<(f64, f64)> {
		let s = Self::size();
		if s.width == 0 || s.height == 0 {
			return None;
		}
		Some((f64::from(s.width) / f64::from(s.columns), f64::from(s.height) / f64::from(s.rows)))
	}

	#[inline]
	pub fn clear(w: &mut impl Write) -> Result<()> {
		queue!(w, Clear(ClearType::All))?;
		writeln!(w)?;
		Ok(w.flush()?)
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
