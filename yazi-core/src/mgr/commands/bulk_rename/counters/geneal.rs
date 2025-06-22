use std::fmt;

/// This macro generates an implementation of CounterFormatter for a given
/// counter helper type.
///
/// # Arguments
///
/// * `$type` - The target helper struct (e.g., `AnsiUpper`).
/// * `$case` - A boolean constant determining whether the counter uppercase or lowercase.
macro_rules! impl_counter_formatter {
	($type:ty, $case:expr) => {
		impl CounterFormatter for $type {
			#[inline]
			fn value_to_buffer(self, value: u32, width: usize, buf: &mut impl fmt::Write) -> fmt::Result {
				write_number_as_letters::<{ $case }>(value, width, buf)
			}

			#[inline]
			fn string_to_value(self, value: &str) -> Option<u32> {
				convert_letters_to_number::<{ $case }>(value)
			}
		}
	};
}

/// Converts a given numeric value into an alphabetic representation following a base-N numbering system,
/// similar to Excel-style column labels (e.g., 1 → A, 2 → B, ..., 26 → Z, 27 → AA, etc.).
///
/// This function generalizes the process for different alphabets by allowing a customizable base (`alphabet_len`)
/// and a transformation function (`convert_fn`) that maps remainder values to characters.
///
/// # Arguments
///
/// * `num` - The numeric value to be converted. Since alphabetic numbering systems start from 1
///   (e.g., A = 1, B = 2), it should be non-zero value.
///
/// * `width` - The minimum width of the output string. If necessary, the result will be left-padded with '0'.
///
/// * `alphabet_len` - The base of the numbering system (e.g., 26 for Latin, 28 for Cyrillic, etc.).
///
/// * `convert_fn` - A closure that converts a remainder (`u32`) into a corresponding character.
///   - The `remainder` represents the remainder of division by `alphabet_len` (i.e., `num % alphabet_len`).
///   - The closure should map this remainder to a specific character in the corresponding alphabet
///     (e.g., `b'A' + remainder as u8`).
///
/// * `buf` - A mutable reference to a `fmt::Write` buffer where the result is written.
#[inline]
pub(super) fn write_number_as_letters_gen(
	mut num: u32,
	width: usize,
	alphabet_len: u32,
	mut convert_fn: impl FnMut(u32) -> char,
	buf: &mut impl fmt::Write,
) -> fmt::Result {
	if num == 0 {
		return Ok(());
	}

	let mut stack_buf = ['0'; 10];
	let mut written_len = 0;

	for char in &mut stack_buf {
		if num == 0 {
			break;
		}
		let remainder = (num - 1) % alphabet_len;
		*char = convert_fn(remainder);
		num = (num - remainder - 1) / alphabet_len;
		written_len += 1;
	}

	if num > 0 {
		let mut vec_buf = Vec::with_capacity(20);
		vec_buf.extend_from_slice(&stack_buf);

		while num > 0 {
			let remainder = (num - 1) % alphabet_len;
			vec_buf.push(convert_fn(remainder));
			num = (num - remainder - 1) / alphabet_len;
			written_len += 1;
		}

		for _ in vec_buf.len()..width {
			buf.write_char('0')?;
		}
		for &c in vec_buf.iter().rev() {
			buf.write_char(c)?;
		}
	} else {
		for _ in written_len..width {
			buf.write_char('0')?;
		}
		for &c in stack_buf[..written_len].iter().rev() {
			buf.write_char(c)?;
		}
	}

	Ok(())
}
