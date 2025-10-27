use std::{ffi::{CStr, OsStr, OsString, c_char, c_void}, mem::ManuallyDrop, os::unix::ffi::OsStrExt, path::PathBuf};

use anyhow::{Result, bail};
use core_foundation_sys::{base::{CFRelease, TCFTypeRef}, dictionary::{CFDictionaryGetValueIfPresent, CFDictionaryRef}, string::CFStringRef};
use objc2::{msg_send, runtime::AnyObject};

use super::cf_string::CFString;

pub struct CFDict(CFDictionaryRef);

impl CFDict {
	pub fn take(dict: CFDictionaryRef) -> Result<Self> {
		if dict.is_null() {
			bail!("Cannot take a null pointer");
		}
		Ok(Self(dict))
	}

	pub fn value(&self, key: &str) -> Result<*const c_void> {
		let key_ = CFString::new(key)?;
		let mut value = std::ptr::null();
		if unsafe { CFDictionaryGetValueIfPresent(self.0, key_.as_void_ptr(), &mut value) } == 0
			|| value.is_null()
		{
			bail!("Cannot get the value for the key `{key}`");
		}
		Ok(value)
	}

	pub fn bool(&self, key: &str) -> Result<bool> {
		let value = self.value(key)?;
		#[allow(unexpected_cfgs)]
		Ok(unsafe { msg_send![value as *const AnyObject, boolValue] })
	}

	pub fn integer(&self, key: &str) -> Result<i64> {
		let value = self.value(key)?;
		#[allow(unexpected_cfgs)]
		Ok(unsafe { msg_send![value as *const AnyObject, longLongValue] })
	}

	pub fn os_string(&self, key: &str) -> Result<OsString> {
		ManuallyDrop::new(CFString(self.value(key)? as CFStringRef)).os_string()
	}

	pub fn path_buf(&self, key: &str) -> Result<PathBuf> {
		let url = self.value(key)? as *const AnyObject;
		#[allow(unexpected_cfgs)]
		let cstr: *const c_char = unsafe {
			let nss: *const AnyObject = msg_send![url, path];
			msg_send![nss, UTF8String]
		};
		Ok(OsStr::from_bytes(unsafe { CStr::from_ptr(cstr) }.to_bytes()).into())
	}
}

impl Drop for CFDict {
	fn drop(&mut self) { unsafe { CFRelease(self.0 as _) } }
}
