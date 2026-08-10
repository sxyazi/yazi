use std::borrow::Cow;

#[cfg(not(windows))]
pub fn safename(name: &[u8]) -> Option<Cow<'_, [u8]>> {
	if matches!(name, b"" | b"." | b"..") {
		None
	} else if !name.iter().any(|&b| matches!(b, 0 | b'/')) {
		Some(Cow::Borrowed(name))
	} else {
		Some(Cow::Owned(name.iter().map(|&b| if matches!(b, 0 | b'/') { b'_' } else { b }).collect()))
	}
}

#[cfg(windows)]
pub fn safename(name: &[u8]) -> Option<Cow<'_, [u8]>> {
	let mut name = Cow::Borrowed(name);

	// Windows disallows trailing spaces and dots in filenames, so we trim them off.
	let len = name.iter().rposition(|&b| b != b' ' && b != b'.').map_or(0, |i| i + 1);
	if len != name.len() {
		name.to_mut().truncate(len);
	}

	// Reject empty names after trimming.
	if name.is_empty() {
		return None;
	}

	for i in 0..name.len() {
		if windows_invalid(name[i]) {
			name.to_mut()[i] = b'_';
		}
	}

	if windows_reserved(&name) {
		name.to_mut().insert(0, b'_');
	}
	Some(name)
}

#[cfg(windows)]
fn windows_invalid(b: u8) -> bool {
	b < 32 || matches!(b, b'<' | b'>' | b':' | b'"' | b'/' | b'\\' | b'|' | b'?' | b'*')
}

#[cfg(windows)]
fn windows_reserved(name: &[u8]) -> bool {
	const NAMES: &[&[u8]] = &[b"CON", b"PRN", b"AUX", b"NUL", b"CONIN$", b"CONOUT$"];

	let stem = name.split(|&b| b == b'.').next().unwrap_or_default().trim_ascii_end();

	if NAMES.iter().any(|name| stem.eq_ignore_ascii_case(name)) {
		true
	} else if stem.len() >= 4 && matches!(&stem[3..], [b'1'..=b'9'] | [0xc2, 0xb2 | 0xb3 | 0xb9]) {
		stem[..3].eq_ignore_ascii_case(b"COM") || stem[..3].eq_ignore_ascii_case(b"LPT")
	} else {
		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rejects_special_components() {
		assert_eq!(safename(b""), None);
		assert_eq!(safename(b"."), None);
		assert_eq!(safename(b".."), None);
		#[cfg(windows)]
		assert_eq!(safename(b". ."), None);
	}

	#[test]
	fn sanitizes_relative_paths() {
		assert_eq!(safename(b"./").as_deref(), Some(&b"._"[..]));
		assert_eq!(safename(b"../").as_deref(), Some(&b".._"[..]));
		assert_eq!(safename(b"./foo").as_deref(), Some(&b"._foo"[..]));
		assert_eq!(safename(b"../foo").as_deref(), Some(&b".._foo"[..]));
	}

	#[test]
	fn replaces_invalid_bytes() {
		#[cfg(not(windows))]
		assert_eq!(safename(b"a\0b/c").as_deref(), Some(&b"a_b_c"[..]));

		#[cfg(windows)]
		for invalid in [0, b'<', b'>', b':', b'"', b'/', b'\\', b'|', b'?', b'*'] {
			let name = [b'a', invalid, b'b'];
			assert_eq!(safename(&name).as_deref(), Some(&b"a_b"[..]));
		}
	}

	#[test]
	fn preserves_valid_names() {
		assert_eq!(safename(b"foo.txt"), Some(Cow::Borrowed(&b"foo.txt"[..])));

		#[cfg(not(windows))]
		assert_eq!(safename(b"CON:a\\b*.txt"), Some(Cow::Borrowed(&b"CON:a\\b*.txt"[..])));

		#[cfg(windows)]
		{
			assert_eq!(safename(b"COM0.txt"), Some(Cow::Borrowed(&b"COM0.txt"[..])));
			assert_eq!(safename(b"COM10.txt"), Some(Cow::Borrowed(&b"COM10.txt"[..])));
			assert_eq!(safename(b"COM4a.txt"), Some(Cow::Borrowed(&b"COM4a.txt"[..])));
		}
	}

	#[cfg(windows)]
	#[test]
	fn trims_trailing_spaces_and_dots() {
		assert_eq!(safename(b"foo.txt. ").as_deref(), Some(&b"foo.txt"[..]));
	}

	#[cfg(windows)]
	#[test]
	fn prefixes_reserved_names() {
		for name in [
			&b"CON"[..],
			&b"prn.txt"[..],
			&b"AUX"[..],
			&b"nul .txt"[..],
			&b"COM1.log"[..],
			&b"lpt9"[..],
			"COM¹.log".as_bytes(),
			"lpt²".as_bytes(),
			"LPT³.txt".as_bytes(),
			&b"CONIN$"[..],
			&b"conout$.txt"[..],
		] {
			let safe = safename(name).unwrap();
			assert_eq!(safe[0], b'_');
			assert_eq!(&safe[1..], name);
		}
	}
}
