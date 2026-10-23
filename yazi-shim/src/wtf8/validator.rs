#[cfg(windows)]
pub(super) fn valid_wtf8(mut bytes: &[u8]) -> bool {
	while let Err(e) = std::str::from_utf8(bytes) {
		// WTF-8 also permits UTF-8 encodings of surrogate code points.
		let [0xed, 0xa0..=0xbf, 0x80..=0xbf, rest @ ..] = &bytes[e.valid_up_to()..] else {
			return false;
		};
		bytes = rest;
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
