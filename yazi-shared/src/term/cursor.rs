use std::io::{stderr, Write};

use anyhow::Result;
use crossterm::{cursor::{MoveTo, SetCursorStyle}, queue};

use super::Term;

impl Term {
	#[inline]
	pub fn move_to(w: &mut impl Write, x: u16, y: u16) -> Result<()> { Ok(queue!(w, MoveTo(x, y))?) }

	#[inline]
	pub fn set_cursor_block() -> Result<()> { Ok(queue!(stderr(), SetCursorStyle::BlinkingBlock)?) }

	#[inline]
	pub fn set_cursor_bar() -> Result<()> { Ok(queue!(stderr(), SetCursorStyle::BlinkingBar)?) }
}
