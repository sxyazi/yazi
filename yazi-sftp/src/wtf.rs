use std::{borrow::Cow, ffi::{OsStr, OsString}, os::windows::ffi::{OsStrExt, OsStringExt}};

pub(super) fn bytes_to_wide(mut bytes: &[u8]) -> Cow<'_, OsStr> {
	let mut wide: Option<Vec<u16>> = None;
	while !bytes.is_empty() {
		match (str::from_utf8(bytes), &mut wide) {
			(Ok(valid), None) => {
				return OsStr::new(valid).into();
			}
			(Ok(valid), Some(wide)) => {
				for ch in valid.chars() {
					wide.extend(ch.encode_utf16(&mut [0u16; 2]).iter());
				}
				break;
			}
			(Err(err), _) => {
				let wide = wide.get_or_insert_with(|| Vec::with_capacity(bytes.len()));

				let valid = unsafe { str::from_utf8_unchecked(&bytes[..err.valid_up_to()]) };
				for c in valid.chars() {
					wide.extend(c.encode_utf16(&mut [0u16; 2]).iter());
				}
				bytes = &bytes[valid.len()..];

				let invalid = err.error_len().unwrap_or(bytes.len());
				for &b in &bytes[..invalid] {
					wide.push(0xdc00 + b as u16);
				}
				bytes = &bytes[invalid..];
			}
		}
	}
	OsString::from_wide(&wide.unwrap_or_default()).into()
}

pub(super) fn wide_to_bytes(wide: &OsStr) -> Option<Cow<'_, [u8]>> {
	if let Some(s) = wide.to_str() {
		return Some(s.as_bytes().into());
	}

	let mut it = wide.encode_wide();
	let mut out = Vec::with_capacity(wide.len());

	while let Some(w) = it.next() {
		if (0xdc00..=0xdcff).contains(&w) {
			out.push((w - 0xdc00) as u8);
		} else if (0xd800..=0xdbff).contains(&w) {
			let x = it.next().filter(|x| (0xdc00..=0xdfff).contains(x))?;
			let c = char::from_u32(0x10000 + (((w as u32 - 0xd800) << 10) | (x as u32 - 0xdc00)))?;
			out.extend_from_slice(c.encode_utf8(&mut [0u8; 4]).as_bytes());
		} else if (0xdc00..=0xdfff).contains(&w) {
			return None;
		} else {
			let c = char::from_u32(w as u32)?;
			out.extend_from_slice(c.encode_utf8(&mut [0u8; 4]).as_bytes());
		}
	}

	Some(out.into())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn wtf8_roundtrip() {
		let b = &[
			b'\0', // NUL
			0xff,  // 0xFF
			b'a', b'b', b'c', // abc
			0xf0, 0x9f, 0x98, 0x80, // 😀
			0xc3, 0x28, // illegal UTF-8
		];
		assert_eq!(&*wide_to_bytes(&bytes_to_wide(b)).unwrap(), b);
	}

	#[test]
	#[cfg(windows)]
	fn low_surrogates_for_non_utf8() {
		use std::os::windows::ffi::OsStrExt;

		let os = bytes_to_wide(b"\xFF");
		let wide: Vec<u16> = os.encode_wide().collect();
		assert_eq!(wide, vec![0xdc00 + 0xff]);
	}
}
