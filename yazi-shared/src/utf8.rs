// https://tools.ietf.org/html/rfc3629
const UTF8_CHAR_WIDTH: &[u8; 256] = &[
	// 1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
	0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
	2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
	3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
	4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F
];

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[must_use]
#[inline]
pub const fn utf8_char_width(b: u8) -> usize { UTF8_CHAR_WIDTH[b as usize] as usize }

/// Finds the closest `x` not exceeding `index` where [`is_char_boundary(x)`] is
/// `true`.
///
/// This method can help you truncate a string so that it's still valid UTF-8,
/// but doesn't exceed a given number of bytes. Note that this is done purely at
/// the character level and can still visually split graphemes, even though the
/// underlying characters aren't split. For example, the emoji 🧑‍🔬 (scientist)
/// could be split so that the string only includes 🧑 (person) instead.
#[inline]
pub fn floor_char_boundary(s: &str, index: usize) -> usize {
	if index >= s.len() {
		s.len()
	} else {
		let lower_bound = index.saturating_sub(3);
		let new_index =
			s.as_bytes()[lower_bound..=index].iter().rposition(|&b| is_utf8_char_boundary(b));

		// SAFETY: we know that the character boundary will be within four bytes
		unsafe { lower_bound + new_index.unwrap_unchecked() }
	}
}

#[inline]
const fn is_utf8_char_boundary(b: u8) -> bool {
	// This is bit magic equivalent to: b < 128 || b >= 192
	(b as i8) >= -0x40
}
