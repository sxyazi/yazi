//! Escape characters that may have special meaning in a shell, including
//! spaces. This is a modified version of the [`shell-escape`] crate and [`this
//! PR`].
//!
//! [`shell-escape`]: https://crates.io/crates/shell-escape
//! [`this PR`]: https://github.com/sfackler/shell-escape/pull/9

use std::{borrow::Cow, ffi::OsStr};

mod unix;
mod windows;

#[inline]
pub fn unix(s: &str) -> Cow<str> { unix::from_str(s) }

#[inline]
pub fn windows(s: &str) -> Cow<str> { windows::from_str(s) }

#[inline]
pub fn native(s: &str) -> Cow<str> {
	#[cfg(unix)]
	{
		unix::from_str(s)
	}
	#[cfg(windows)]
	{
		windows::from_str(s)
	}
}

#[inline]
pub fn os_str(s: &OsStr) -> Cow<OsStr> {
	#[cfg(unix)]
	{
		unix::from_os_str(s)
	}
	#[cfg(windows)]
	{
		windows::from_os_str(s)
	}
}
