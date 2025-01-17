use std::{ffi::OsString, ops::Deref, os::unix::ffi::OsStringExt};

use anyhow::{Result, bail};
use core_foundation_sys::{base::{CFRelease, kCFAllocatorDefault, kCFAllocatorNull}, string::{CFStringCreateWithBytesNoCopy, CFStringGetCString, CFStringGetLength, CFStringGetMaximumSizeForEncoding, CFStringRef, kCFStringEncodingUTF8}};
use libc::strlen;

pub struct CFString(pub(super) CFStringRef);

impl CFString {
	pub fn new(s: &str) -> Result<Self> {
		let key = unsafe {
			CFStringCreateWithBytesNoCopy(
				kCFAllocatorDefault,
				s.as_ptr(),
				s.len() as _,
				kCFStringEncodingUTF8,
				false as _,
				kCFAllocatorNull,
			)
		};
		if key.is_null() {
			bail!("Allocation failed while creating CFString");
		}
		Ok(Self(key))
	}

	pub fn len(&self) -> usize { unsafe { CFStringGetLength(self.0) as _ } }

	pub fn is_empty(&self) -> bool { self.len() == 0 }

	pub fn os_string(&self) -> Result<OsString> {
		let len = self.len();
		let capacity =
			unsafe { CFStringGetMaximumSizeForEncoding(len as _, kCFStringEncodingUTF8) } + 1;

		let mut out: Vec<u8> = Vec::with_capacity(capacity as usize);
		let result = unsafe {
			CFStringGetCString(self.0, out.as_mut_ptr().cast(), capacity, kCFStringEncodingUTF8)
		};
		if result == 0 {
			bail!("Failed to get the C string from CFString");
		}

		unsafe { out.set_len(strlen(out.as_ptr().cast())) };
		out.shrink_to_fit();
		Ok(OsString::from_vec(out))
	}
}

impl Deref for CFString {
	type Target = CFStringRef;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Drop for CFString {
	fn drop(&mut self) { unsafe { CFRelease(self.0 as _) }; }
}
