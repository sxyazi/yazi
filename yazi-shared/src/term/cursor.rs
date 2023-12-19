use std::io::{stdout, Write};

use anyhow::Result;
use crossterm::{cursor::{MoveTo, RestorePosition, SavePosition, SetCursorStyle}, execute, queue};

use super::Term;

impl Term {
	#[inline]
	pub fn move_to(stdout: &mut impl Write, x: u16, y: u16) -> Result<()> {
		Ok(queue!(stdout, MoveTo(x, y))?)
	}

	#[inline]
	pub fn move_lock<W, F, T>(mut stdout: W, (x, y): (u16, u16), cb: F) -> Result<T>
	where
		W: Write,
		F: FnOnce(&mut W) -> Result<T>,
	{
		#[cfg(windows)]
		{
			use std::{thread, time::Duration};

			use crossterm::cursor::{Hide, Show};
			queue!(&mut stdout, SavePosition, MoveTo(x, y), Show)?;

			// I really don't want to add this,
			// but on Windows the cursor position will not synchronize in time occasionally
			stdout.flush()?;
			thread::sleep(Duration::from_millis(1));

			let result = cb(&mut stdout);
			queue!(&mut stdout, Hide, RestorePosition)?;
			stdout.flush()?;
			result
		}
		#[cfg(unix)]
		{
			queue!(&mut stdout, SavePosition, MoveTo(x, y))?;
			let result = cb(&mut stdout);
			queue!(&mut stdout, RestorePosition)?;
			stdout.flush()?;
			result
		}
	}

	#[inline]
	pub fn set_cursor_block() -> Result<()> { Ok(execute!(stdout(), SetCursorStyle::BlinkingBlock)?) }

	#[inline]
	pub fn set_cursor_bar() -> Result<()> { Ok(execute!(stdout(), SetCursorStyle::BlinkingBar)?) }

	#[inline]
	pub fn set_cursor_default() -> Result<()> {
		Ok(execute!(stdout(), SetCursorStyle::DefaultUserShape)?)
	}
}
