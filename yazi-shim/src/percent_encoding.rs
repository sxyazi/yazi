use std::fmt;

use percent_encoding::{AsciiSet, percent_encode};

// --- PercentEncoder
pub struct PercentEncoder<'a, W: ?Sized> {
	writer: &'a mut W,
	set:    &'static AsciiSet,
}

impl<'a, W: ?Sized> PercentEncoder<'a, W> {
	pub fn new(writer: &'a mut W, set: &'static AsciiSet) -> Self { Self { writer, set } }
}

impl<W: fmt::Write + ?Sized> fmt::Write for PercentEncoder<'_, W> {
	fn write_str(&mut self, value: &str) -> fmt::Result {
		for chunk in percent_encode(value.as_bytes(), self.set) {
			fmt::Write::write_str(self.writer, chunk)?;
		}
		Ok(())
	}
}
