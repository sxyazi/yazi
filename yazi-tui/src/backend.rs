use std::{io::{self, Write}, ops::{Deref, DerefMut}};

use ratatui::{backend::{Backend, ClearType, WindowSize}, buffer::Cell, layout::{Position, Size}, style::{Color, Modifier}};
use yazi_macro::writef;
use yazi_term::{TERM, sequence::{EraseRegion, HideCursor, MoveTo, ResetAttrs, SetBg, SetFg, SetSgr, SetUnderlineColor, ShowCursor}};

pub struct RatermBackend<W> {
	writer:     W,
	cursor_pos: Position,
}

impl<W> RatermBackend<W> {
	pub fn new(writer: W) -> Self { Self { writer, cursor_pos: Position::default() } }
}

impl<W> Deref for RatermBackend<W> {
	type Target = W;

	fn deref(&self) -> &Self::Target { &self.writer }
}

impl<W> DerefMut for RatermBackend<W> {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.writer }
}

impl<W> Backend for RatermBackend<W>
where
	W: Write,
{
	type Error = io::Error;

	fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
	where
		I: Iterator<Item = (u16, u16, &'a Cell)>,
	{
		let mut fg = Color::Reset;
		let mut bg = Color::Reset;
		let mut underline_color = Color::Reset;
		let mut modifier = Modifier::empty();
		let mut last_pos: Option<Position> = None;

		for (x, y, cell) in content {
			// Move the cursor if the previous location was not (x - 1, y)
			if !matches!(last_pos, Some(p) if x == p.x + 1 && y == p.y) {
				write!(self.writer, "{}", MoveTo(x, y))?;
			}
			last_pos = Some(Position { x, y });

			if cell.modifier != modifier {
				write_modifier_diff(&mut self.writer, modifier, cell.modifier)?;
				modifier = cell.modifier;
			}

			if cell.fg != fg {
				write!(self.writer, "{}", SetFg(cell.fg))?;
				fg = cell.fg;
			}

			if cell.bg != bg {
				write!(self.writer, "{}", SetBg(cell.bg))?;
				bg = cell.bg;
			}

			if cell.underline_color != underline_color {
				write!(self.writer, "{}", SetUnderlineColor(cell.underline_color))?;
				underline_color = cell.underline_color;
			}

			write!(self.writer, "{}", cell.symbol())?;
		}

		write!(self.writer, "{ResetAttrs}")
	}

	fn hide_cursor(&mut self) -> io::Result<()> { write!(self.writer, "{HideCursor}") }

	fn show_cursor(&mut self) -> io::Result<()> { write!(self.writer, "{ShowCursor}") }

	fn get_cursor_position(&mut self) -> io::Result<Position> { Ok(self.cursor_pos) }

	fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
		let Position { x, y } = position.into();
		write!(self.writer, "{}", MoveTo(x, y))?;
		self.cursor_pos = Position { x, y };
		self.writer.flush()
	}

	fn clear(&mut self) -> io::Result<()> { self.clear_region(ClearType::All) }

	fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
		writef!(self.writer, "{}", EraseRegion(clear_type))
	}

	fn append_lines(&mut self, n: u16) -> io::Result<()> {
		for _ in 0..n {
			writeln!(self.writer)?;
		}
		self.writer.flush()
	}

	fn size(&self) -> io::Result<Size> {
		let dim = TERM.dimension().checked()?;
		Ok(Size { width: dim.cols, height: dim.rows })
	}

	fn window_size(&mut self) -> io::Result<WindowSize> {
		let dim = TERM.dimension().checked()?;
		Ok(WindowSize {
			columns_rows: Size { width: dim.cols, height: dim.rows },
			pixels:       Size { width: dim.width, height: dim.height },
		})
	}

	fn flush(&mut self) -> io::Result<()> { self.writer.flush() }
}

/// Apply the diff between two `Modifier` sets by emitting SGR codes.
fn write_modifier_diff(w: &mut impl Write, from: Modifier, to: Modifier) -> io::Result<()> {
	let removed = from - to;
	let added = to - from;

	// Process removed styles
	let removed_styles = [
		(Modifier::REVERSED, SetSgr::NoReverse),
		(Modifier::ITALIC, SetSgr::NoItalic),
		(Modifier::UNDERLINED, SetSgr::NoUnderline),
		(Modifier::CROSSED_OUT, SetSgr::NotCrossedOut),
		(Modifier::HIDDEN, SetSgr::NoHidden),
	];
	for (modifier, sgr) in removed_styles {
		if removed.contains(modifier) {
			write!(w, "{}", sgr)?;
		}
	}

	if removed.contains(Modifier::SLOW_BLINK) || removed.contains(Modifier::RAPID_BLINK) {
		write!(w, "{}", SetSgr::NoBlink)?;
	}

	// Process intensity (Bold / Dim) status transitions
	let reset_intensity = removed.contains(Modifier::BOLD) || removed.contains(Modifier::DIM);
	if reset_intensity {
		// Bold and Dim are both reset by applying the Normal intensity
		write!(w, "{}", SetSgr::NormalIntensity)?;

		// The remaining Bold and Dim attributes must be
		// reapplied after the intensity reset above.
		if to.contains(Modifier::DIM) {
			write!(w, "{}", SetSgr::Dim)?;
		}
		if to.contains(Modifier::BOLD) {
			write!(w, "{}", SetSgr::Bold)?;
		}
	} else {
		if added.contains(Modifier::BOLD) {
			write!(w, "{}", SetSgr::Bold)?;
		}
		if added.contains(Modifier::DIM) {
			write!(w, "{}", SetSgr::Dim)?;
		}
	}

	// Process added styles
	let added_styles = [
		(Modifier::REVERSED, SetSgr::Reverse),
		(Modifier::ITALIC, SetSgr::Italic),
		(Modifier::UNDERLINED, SetSgr::Underlined),
		(Modifier::CROSSED_OUT, SetSgr::CrossedOut),
		(Modifier::HIDDEN, SetSgr::Hidden),
		(Modifier::SLOW_BLINK, SetSgr::SlowBlink),
		(Modifier::RAPID_BLINK, SetSgr::RapidBlink),
	];
	for (modifier, sgr) in added_styles {
		if added.contains(modifier) {
			write!(w, "{}", sgr)?;
		}
	}

	Ok(())
}
