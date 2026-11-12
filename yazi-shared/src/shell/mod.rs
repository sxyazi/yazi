//! Escape characters that may have special meaning in a shell, including
//! spaces. This is a modified version of the [`shell-escape`], [`shell-words`],
//! [Rust std] and [this PR].
//!
//! [`shell-escape`]: https://crates.io/crates/shell-escape
//! [`shell-words`]: https://crates.io/crates/shell-words
//! [Rust std]: https://github.com/rust-lang/rust/blob/main/library/std/src/sys/args/windows.rs#L220
//! [this PR]: https://github.com/sfackler/shell-escape/pull/9

yazi_macro::mod_pub!(unix windows);

yazi_macro::mod_flat!(error);

#[cfg(unix)]
pub use unix::{escape_os_bytes, escape_os_bytes_into};
#[cfg(windows)]
pub use windows::{escape_os_bytes, escape_os_bytes_into};
