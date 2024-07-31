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
pub fn escape_unix(s: &str) -> Cow<str> { unix::escape_str(s) }

#[inline]
pub fn escape_windows(s: &str) -> Cow<str> { windows::escape_str(s) }

#[inline]
pub fn escape_native(s: &str) -> Cow<str> {
	#[cfg(unix)]
	{
		escape_unix(s)
	}
	#[cfg(windows)]
	{
		escape_windows(s)
	}
}

#[inline]
pub fn escape_os_str(s: &OsStr) -> Cow<OsStr> {
	#[cfg(unix)]
	{
		unix::escape_os_str(s)
	}
	#[cfg(windows)]
	{
		windows::escape_os_str(s)
	}
}

#[inline]
pub fn split_unix(s: &str) -> anyhow::Result<Vec<String>> { Ok(shell_words::split(s)?) }

#[cfg(windows)]
pub fn split_windows(s: &str) -> anyhow::Result<Vec<String>> { Ok(windows::split(s)?) }

pub fn split_native(s: &str) -> anyhow::Result<Vec<String>> {
	#[cfg(unix)]
	{
		split_unix(s)
	}
	#[cfg(windows)]
	{
		split_windows(s)
	}
}
