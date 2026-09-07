use std::fmt;

use percent_encoding::{AsciiSet, CONTROLS, percent_encode};

// RFC 3986 path component: encode everything that is not a safe path character.
// Safe chars: unreserved (A-Za-z0-9 -._~) + sub-delims (!$&'()*+,;=) + : @ /
pub const RFC_3986: &AsciiSet = &CONTROLS
	.add(b' ')
	.add(b'"')
	.add(b'#')
	.add(b'%')
	.add(b'<')
	.add(b'>')
	.add(b'?')
	.add(b'[')
	.add(b'\\')
	.add(b']')
	.add(b'^')
	.add(b'`')
	.add(b'{')
	.add(b'|')
	.add(b'}');

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
