#[cfg(windows)]
pub(super) fn valid_wtf8(bytes: &[u8]) -> bool {
	let mut i = 0;
	while i < bytes.len() {
		let b = bytes[i];
		if b < 0b1000_0000 {
			// ASCII
			i += 1;
			continue;
		}

		// 2-byte: 110x_xxxx 10xx_xxxx
		// first byte must be >= 0b1100_0010 to forbid overlongs
		if (b & 0b1110_0000) == 0b1100_0000 {
			if b < 0b1100_0010 {
				return false;
			}
			if i + 1 >= bytes.len() {
				return false;
			}
			if (bytes[i + 1] & 0b1100_0000) != 0b1000_0000 {
				return false;
			}
			i += 2;
			continue;
		}

		// 3-byte: 1110_xxxx 10xx_xxxx 10xx_xxxx
		if (b & 0b1111_0000) == 0b1110_0000 {
			if i + 2 >= bytes.len() {
				return false;
			}
			let (b1, b2) = (bytes[i + 1], bytes[i + 2]);
			if (b1 & 0b1100_0000) != 0b1000_0000 || (b2 & 0b1100_0000) != 0b1000_0000 {
				return false;
			}
			if b == 0b1110_0000 && b1 < 0b1010_0000 {
				// to forbid overlongs, second byte must be >= 0xA0
				return false;
			}
			i += 3;
			continue;
		}

		// 4-byte: 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
		if (b & 0b1111_1000) == 0b1111_0000 {
			if b > 0b1111_0100 {
				return false;
			}
			if i + 3 >= bytes.len() {
				return false;
			}
			let (b1, b2, b3) = (bytes[i + 1], bytes[i + 2], bytes[i + 3]);
			if (b1 & 0b1100_0000) != 0b1000_0000
				|| (b2 & 0b1100_0000) != 0b1000_0000
				|| (b3 & 0b1100_0000) != 0b1000_0000
			{
				return false;
			}
			if b == 0b1111_0000 && b1 < 0b1001_0000 {
				// to forbid overlongs for > U+FFFF, second byte must be >= 0x90
				return false;
			} else if b == 0b1111_0100 && b1 > 0b1000_1111 {
				// to stay <= U+10FFFF, second byte must be <= 0x8F
				return false;
			}
			i += 4;
			continue;
		}

		return false;
	}
	true
}

#[cfg(windows)]
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_valid_wtf8() {
		let cases: &[(&[u8], bool)] = &[
			// Valid ASCII
			(b"hello", true),
			// Valid 2-byte UTF-8
			(&[0xc2, 0xa0], true), // U+00A0
			// Invalid 2-byte: overlong encoding
			(&[0xc0, 0x80], false), // overlong for U+0000
			(&[0xc1, 0xbf], false), // overlong for U+007F
			// Valid 3-byte UTF-8
			(&[0xe0, 0xa0, 0x80], true), // U+0800
			// Invalid 3-byte: overlong encoding
			(&[0xe0, 0x9f, 0xbf], false), // overlong for U+07FF
			// WTF-8 specific: unpaired surrogates (should be valid in WTF-8)
			((&[0xed, 0xa0, 0x80]), true), // U+D800 = ED A0 80 (high surrogate)
			((&[0xed, 0xbf, 0xbf]), true), // U+DFFF = ED BF BF (low surrogate)
			// Valid 4-byte UTF-8
			((&[0xf0, 0x90, 0x80, 0x80]), true), // U+10000
			// Invalid 4-byte: overlong
			((&[0xf0, 0x8f, 0xbf, 0xbf]), false), // overlong for U+FFFF
			// Invalid 4-byte: beyond U+10FFFF
			((&[0xf4, 0x90, 0x80, 0x80]), false), // U+110000
			// Invalid continuation byte
			((&[0xc2, 0x00]), false),
			// Incomplete sequence
			((&[0xc2]), false),
			((&[0xe0, 0xa0]), false),
			((&[0xf0, 0x90, 0x80]), false),
		];

		for &(input, expected) in cases {
			assert_eq!(valid_wtf8(input), expected, "input: {:?}", input);
		}
	}
}
