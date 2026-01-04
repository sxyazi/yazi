//! Escape characters that may have special meaning in a shell, including
//! spaces. This is a modified version of the [`shell-escape`], [`shell-words`],
//! [Rust std] and [this PR].
//!
//! [`shell-escape`]: https://crates.io/crates/shell-escape
//! [`shell-words`]: https://crates.io/crates/shell-words
//! [Rust std]: https://github.com/rust-lang/rust/blob/main/library/std/src/sys/args/windows.rs#L220
//! [this PR]: https://github.com/sfackler/shell-escape/pull/9

use std::{borrow::Cow, ffi::OsStr};

yazi_macro::mod_pub!(unix, windows);

yazi_macro::mod_flat!(error);

#[inline]
pub fn escape_os_bytes(b: &[u8]) -> Cow<'_, [u8]> {
	#[cfg(unix)]
	{
		unix::escape_os_bytes(b)
	}
	#[cfg(windows)]
	{
		windows::escape_os_bytes(b)
	}
}

#[inline]
pub fn escape_os_str(s: &OsStr) -> Cow<'_, OsStr> {
	#[cfg(unix)]
	{
		unix::escape_os_str(s)
	}
	#[cfg(windows)]
	{
		windows::escape_os_str(s)
	}
}
