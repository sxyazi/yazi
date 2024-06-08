use std::io::Write;

use crossterm::queue;

#[inline]
pub fn terminal_clear(w: &mut impl Write) -> std::io::Result<()> {
	queue!(w, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
	writeln!(w)?;
	w.flush()
}
