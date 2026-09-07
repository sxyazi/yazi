use std::fmt;

use super::twiddle::Twiddle;

pub(super) struct TwiddleWriter<'a, W: ?Sized>(pub(super) &'a mut W);

impl<W: fmt::Write + ?Sized> fmt::Write for TwiddleWriter<'_, W> {
	fn write_str(&mut self, value: &str) -> fmt::Result {
		let mut i = 0;
		for (j, b) in value.bytes().enumerate() {
			if Twiddle::len_escape(b) == 1 {
				continue;
			}

			self.0.write_str(&value[i..j])?;
			Twiddle::encode_byte(self.0, b)?;
			i = j + 1;
		}
		self.0.write_str(&value[i..])
	}
}
