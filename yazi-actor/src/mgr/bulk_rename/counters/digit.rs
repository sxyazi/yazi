//! This module provides functionality for managing Arabic numeral counters.

use super::CounterFormatter;
use std::fmt;

/// A helper structure for generating numeric values (e.g., 1, 2, ..., 999 or 001, 002).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Digits;

impl CounterFormatter for Digits {
	/// Formats a value as a zero-padded string and writes it to a buffer.
	///
	/// # Arguments
	///
	/// * `value` - The numeric value to format.
	/// * `width` - The minimum width of the output string.
	/// * `buf` - A mutable reference to a buffer.
	#[inline]
	fn value_to_buffer(
		self,
		value: u32,
		width: usize,
		buf: &mut impl fmt::Write,
	) -> Result<(), fmt::Error> {
		write!(buf, "{value:0>width$}")
	}

	/// Parses a zero-padded numeric string into a `u32` value.
	#[inline]
	fn string_to_value(self, value: &str) -> Option<u32> {
		value.parse().ok()
	}
}
