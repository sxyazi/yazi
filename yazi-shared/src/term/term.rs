use std::{io::{stdout, Stdout, Write}, mem, ops::{Deref, DerefMut}};

use anyhow::Result;
use crossterm::{event::{DisableBracketedPaste, DisableFocusChange, EnableBracketedPaste, EnableFocusChange, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags}, execute, queue, terminal::{disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, WindowSize}};
use ratatui::{backend::CrosstermBackend, Terminal};

pub struct Term {
	inner: Terminal<CrosstermBackend<Stdout>>,
	csi_u: bool,
}

impl Term {
	pub fn start() -> Result<Self> {
		let mut term = Self { inner: Terminal::new(CrosstermBackend::new(stdout()))?, csi_u: false };

		enable_raw_mode()?;
		execute!(stdout(), EnterAlternateScreen, EnableBracketedPaste, EnableFocusChange)?;

		term.csi_u = matches!(supports_keyboard_enhancement(), Ok(true));
		if term.csi_u {
			queue!(
				stdout(),
				PushKeyboardEnhancementFlags(
					KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
						| KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
				)
			)?;
		}

		term.hide_cursor()?;
		term.clear()?;
		Ok(term)
	}

	fn stop(&mut self) -> Result<()> {
		if self.csi_u {
			execute!(stdout(), PopKeyboardEnhancementFlags)?;
		}

		execute!(stdout(), DisableFocusChange, DisableBracketedPaste, LeaveAlternateScreen)?;

		Self::set_cursor_default()?;
		self.show_cursor()?;
		Ok(disable_raw_mode()?)
	}

	pub fn goodbye(f: impl FnOnce() -> bool) -> Result<()> {
		execute!(
			stdout(),
			PopKeyboardEnhancementFlags,
			DisableFocusChange,
			DisableBracketedPaste,
			LeaveAlternateScreen,
			crossterm::cursor::SetCursorStyle::DefaultUserShape,
			crossterm::cursor::Show,
		)?;
		disable_raw_mode()?;
		std::process::exit(f() as i32);
	}

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
	pub fn clear(stdout: &mut impl Write) -> Result<()> {
		queue!(stdout, Clear(ClearType::All))?;
		writeln!(stdout)?;
		Ok(stdout.flush()?)
	}
}

impl Drop for Term {
	fn drop(&mut self) { self.stop().ok(); }
}

impl Deref for Term {
	type Target = Terminal<CrosstermBackend<Stdout>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Term {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}
