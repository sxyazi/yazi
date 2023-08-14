use std::{io::{stdout, Stdout, Write}, ops::{Deref, DerefMut}};

use anyhow::Result;
use crossterm::{cursor::{MoveTo, SetCursorStyle}, event::{DisableBracketedPaste, DisableFocusChange, EnableBracketedPaste, EnableFocusChange, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags}, execute, queue, terminal::{disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}};
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

	#[inline]
	pub fn clear(stdout: &mut impl Write) -> Result<()> {
		execute!(stdout, Clear(ClearType::All))?;
		writeln!(stdout)?;
		Ok(stdout.flush()?)
	}

	#[inline]
	pub fn move_to(stdout: &mut impl Write, x: u16, y: u16) -> Result<()> {
		Ok(execute!(stdout, MoveTo(x, y))?)
	}

	#[inline]
	pub fn set_cursor_block() -> Result<()> { Ok(execute!(stdout(), SetCursorStyle::BlinkingBlock)?) }

	#[inline]
	pub fn set_cursor_bar() -> Result<()> { Ok(execute!(stdout(), SetCursorStyle::BlinkingBar)?) }
}

impl Drop for Term {
	fn drop(&mut self) {
		let mut f = || -> Result<()> {
			if self.csi_u {
				execute!(stdout(), PopKeyboardEnhancementFlags)?;
			}

			execute!(stdout(), DisableFocusChange, DisableBracketedPaste, LeaveAlternateScreen)?;

			self.show_cursor()?;
			Ok(disable_raw_mode()?)
		};

		f().ok();
	}
}

impl Deref for Term {
	type Target = Terminal<CrosstermBackend<Stdout>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Term {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}
