use std::io::Write;

use crossterm::execute;

#[inline]
pub fn terminal_clear(mut w: impl Write) -> std::io::Result<()> {
	execute!(
		w,
		crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
		crossterm::style::Print("\n")
	)
}
