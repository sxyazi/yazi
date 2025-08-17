use std::{borrow::Cow, ffi::{OsStr, OsString}};

use crate::url::UrlBuf;

pub trait IntoStringLossy {
	fn into_string_lossy(self) -> String;
}

impl IntoStringLossy for String {
	fn into_string_lossy(self) -> String { self }
}

impl IntoStringLossy for &OsStr {
	fn into_string_lossy(self) -> String { self.to_string_lossy().into_owned() }
}

impl IntoStringLossy for OsString {
	fn into_string_lossy(self) -> String {
		match self.to_string_lossy() {
			Cow::Owned(s) => s,
			Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(self.into_encoded_bytes()) },
		}
	}
}

impl IntoStringLossy for Cow<'_, OsStr> {
	fn into_string_lossy(self) -> String {
		match self {
			Cow::Owned(s) => s.into_string_lossy(),
			Cow::Borrowed(s) => s.into_string_lossy(),
		}
	}
}

impl IntoStringLossy for &UrlBuf {
	fn into_string_lossy(self) -> String { self.os_str().into_string_lossy() }
}
