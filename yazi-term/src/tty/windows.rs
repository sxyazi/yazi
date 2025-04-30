// Copied from https://github.com/rust-lang/rust/blob/master/library/std/src/sys/pal/windows/stdio.rs

use std::{mem::MaybeUninit, os::windows::io::RawHandle, str};

use windows_sys::Win32::{Globalization::{CP_UTF8, MB_ERR_INVALID_CHARS, MultiByteToWideChar}, System::Console::WriteConsoleW};
use yazi_shared::{floor_char_boundary, utf8_char_width};

// Apparently Windows doesn't handle large reads on stdin or writes to
// stdout/stderr well (see #13304 for details).
//
// From MSDN (2011): "The storage for this buffer is allocated from a shared
// heap for the process that is 64 KB in size. The maximum size of the buffer
// will depend on heap usage."
//
// We choose the cap at 8 KiB because libuv does the same, and it seems to be
// acceptable so far.
const MAX_BUFFER_SIZE: usize = 8192;

#[derive(Default)]
pub(super) struct IncompleteUtf8 {
	bytes: [u8; 4],
	len:   u8,
}

pub(super) fn write_console_utf16(
	data: &[u8],
	incomplete_utf8: &mut IncompleteUtf8,
	handle: RawHandle,
) -> std::io::Result<usize> {
	if incomplete_utf8.len > 0 {
		assert!(incomplete_utf8.len < 4, "Unexpected number of bytes for incomplete UTF-8 codepoint.");
		if data[0] >> 6 != 0b10 {
			// not a continuation byte - reject
			incomplete_utf8.len = 0;
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				"Windows stdio in console mode does not support writing non-UTF-8 byte sequences",
			));
		}
		incomplete_utf8.bytes[incomplete_utf8.len as usize] = data[0];
		incomplete_utf8.len += 1;
		let char_width = utf8_char_width(incomplete_utf8.bytes[0]);
		if (incomplete_utf8.len as usize) < char_width {
			// more bytes needed
			return Ok(1);
		}
		let s = str::from_utf8(&incomplete_utf8.bytes[0..incomplete_utf8.len as usize]);
		incomplete_utf8.len = 0;
		match s {
			Ok(s) => {
				assert_eq!(char_width, s.len());
				let written = write_valid_utf8_to_console(handle, s)?;
				assert_eq!(written, s.len()); // guaranteed by write_valid_utf8_to_console() for single codepoint writes
				return Ok(1);
			}
			Err(_) => {
				return Err(std::io::Error::new(
					std::io::ErrorKind::InvalidData,
					"Windows stdio in console mode does not support writing non-UTF-8 byte sequences",
				));
			}
		}
	}

	// As the console is meant for presenting text, we assume bytes of `data` are
	// encoded as UTF-8, which needs to be encoded as UTF-16.
	//
	// If the data is not valid UTF-8 we write out as many bytes as are valid.
	// If the first byte is invalid it is either first byte of a multi-byte sequence
	// but the provided byte slice is too short or it is the first byte of an
	// invalid multi-byte sequence.
	let len = std::cmp::min(data.len(), MAX_BUFFER_SIZE / 2);
	let utf8 = match str::from_utf8(&data[..len]) {
		Ok(s) => s,
		Err(ref e) if e.valid_up_to() == 0 => {
			let first_byte_char_width = utf8_char_width(data[0]);
			if first_byte_char_width > 1 && data.len() < first_byte_char_width {
				incomplete_utf8.bytes[0] = data[0];
				incomplete_utf8.len = 1;
				return Ok(1);
			} else {
				return Err(std::io::Error::new(
					std::io::ErrorKind::InvalidData,
					"Windows stdio in console mode does not support writing non-UTF-8 byte sequences",
				));
			}
		}
		Err(e) => str::from_utf8(&data[..e.valid_up_to()]).unwrap(),
	};

	write_valid_utf8_to_console(handle, utf8)
}

fn write_valid_utf8_to_console(handle: RawHandle, utf8: &str) -> std::io::Result<usize> {
	debug_assert!(!utf8.is_empty());

	let mut utf16 = [MaybeUninit::<u16>::uninit(); MAX_BUFFER_SIZE / 2];
	let utf8 = &utf8[..floor_char_boundary(utf8, utf16.len())];

	let utf16: &[u16] = unsafe {
		// Note that this theoretically checks validity twice in the (most common) case
		// where the underlying byte sequence is valid utf-8 (given the check in
		// `write()`).
		let result = MultiByteToWideChar(
			CP_UTF8,
			MB_ERR_INVALID_CHARS,
			utf8.as_ptr(),
			utf8.len() as i32,
			utf16.as_mut_ptr() as *mut _,
			utf16.len() as i32,
		);
		assert!(result != 0, "Unexpected error in MultiByteToWideChar");

		// Safety: MultiByteToWideChar initializes `result` values.
		&*(&utf16[..result as usize] as *const [MaybeUninit<u16>] as *const [u16])
	};

	let mut written = write_u16s(handle, utf16)?;

	// Figure out how many bytes of as UTF-8 were written away as UTF-16.
	if written == utf16.len() {
		Ok(utf8.len())
	} else {
		// Make sure we didn't end up writing only half of a surrogate pair (even though
		// the chance is tiny). Because it is not possible for user code to re-slice
		// `data` in such a way that a missing surrogate can be produced (and also
		// because of the UTF-8 validation above), write the missing surrogate out
		// now. Buffering it would mean we have to lie about the number of bytes
		// written.
		let first_code_unit_remaining = utf16[written];
		if matches!(first_code_unit_remaining, 0xdcee..=0xdfff) {
			// low surrogate
			// We just hope this works, and give up otherwise
			let _ = write_u16s(handle, &utf16[written..written + 1]);
			written += 1;
		}
		// Calculate the number of bytes of `utf8` that were actually written.
		let mut count = 0;
		for ch in utf16[..written].iter() {
			count += match ch {
				0x0000..=0x007f => 1,
				0x0080..=0x07ff => 2,
				0xdcee..=0xdfff => 1, // Low surrogate. We already counted 3 bytes for the other.
				_ => 3,
			};
		}
		debug_assert!(String::from_utf16(&utf16[..written]).unwrap() == utf8[..count]);
		Ok(count)
	}
}

fn write_u16s(handle: RawHandle, data: &[u16]) -> std::io::Result<usize> {
	debug_assert!(data.len() < u32::MAX as usize);
	let mut written = 0;
	let result = unsafe {
		WriteConsoleW(handle, data.as_ptr(), data.len() as u32, &mut written, std::ptr::null_mut())
	};
	if result == 0 { Err(std::io::Error::last_os_error()) } else { Ok(written as usize) }
}
