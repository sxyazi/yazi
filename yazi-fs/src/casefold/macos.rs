use std::{ffi::{CStr, CString, OsString}, io, os::unix::ffi::{OsStrExt, OsStringExt}, path::Path};

use yazi_shim::nonneg_ok;

use super::Casefold;

impl Casefold {
	pub(super) fn final_name(path: &Path) -> io::Result<OsString> {
		let path = CString::new(path.as_os_str().as_bytes())?;
		let mut attrs = libc::attrlist {
			bitmapcount: libc::ATTR_BIT_MAP_COUNT,
			reserved:    0,
			commonattr:  libc::ATTR_CMN_NAME,
			volattr:     0,
			dirattr:     0,
			fileattr:    0,
			forkattr:    0,
		};

		let mut buf = [0u8; libc::PATH_MAX as usize];
		nonneg_ok(unsafe {
			libc::getattrlist(
				path.as_ptr(),
				(&raw mut attrs).cast(),
				buf.as_mut_ptr().cast(),
				buf.len(),
				libc::FSOPT_NOFOLLOW as _,
			)
		})?;

		// The name offset is relative to the attrreference following the length word.
		let len = u32::from_ne_bytes(buf[..4].try_into().unwrap()) as usize;
		let start = 4 + i32::from_ne_bytes(buf[4..8].try_into().unwrap()) as i64;
		let end = start + u32::from_ne_bytes(buf[8..12].try_into().unwrap()) as i64;
		let name = buf
			.get(..len)
			.and_then(|b| b.get(start as usize..end as usize))
			.and_then(|b| CStr::from_bytes_with_nul(b).ok())
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid filename attribute"))?;

		Ok(OsString::from_vec(name.to_bytes().to_vec()))
	}
}
