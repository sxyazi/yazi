use std::io::{stderr, BufWriter, StderrLock, Write};

use anyhow::Result;
use crossterm::{cursor::{MoveTo, RestorePosition, SavePosition, SetCursorStyle}, queue};

use super::Term;

impl Term {
	#[inline]
	pub fn move_to(w: &mut impl Write, x: u16, y: u16) -> Result<()> { Ok(queue!(w, MoveTo(x, y))?) }

	// FIXME: remove this function
	#[inline]
	pub fn move_lock<F, T>((x, y): (u16, u16), cb: F) -> Result<T>
	where
		F: FnOnce(&mut BufWriter<StderrLock>) -> Result<T>,
	{
		let mut buf = BufWriter::new(stderr().lock());
		#[cfg(windows)]
		{
			use std::{thread, time::Duration};

			use crossterm::cursor::{Hide, Show};
			queue!(buf, SavePosition, MoveTo(x, y), Show)?;

			// I really don't want to add this,
			// but on Windows the cursor position will not synchronize in time occasionally
			buf.flush()?;
			thread::sleep(Duration::from_millis(1));

			let result = cb(&mut buf);
			queue!(buf, Hide, RestorePosition)?;
			buf.flush()?;
			result
		}
		#[cfg(unix)]
		{
			queue!(buf, SavePosition, MoveTo(x, y))?;
			let result = cb(&mut buf);
			queue!(buf, RestorePosition)?;
			buf.flush()?;
			result
		}
	}

	#[inline]
	pub fn set_cursor_block() -> Result<()> { Ok(queue!(stderr(), SetCursorStyle::BlinkingBlock)?) }

	#[inline]
	pub fn set_cursor_bar() -> Result<()> { Ok(queue!(stderr(), SetCursorStyle::BlinkingBar)?) }
}
