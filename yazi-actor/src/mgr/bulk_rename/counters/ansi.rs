//! This module provides functionality for managing ANSI letter counters for both
//! uppercase and lowercase letters, following Excel's alphabetic counter style.

use super::{CounterFormatter, LOWERCASE, UPPERCASE, write_number_as_letters_gen};
use std::fmt;

/// A helper structure for generating uppercase ANSI letters (e.g., A, B, ..., AA, AB).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnsiUpper;

/// A helper structure for generating lowercase ANSI letters (e.g., a, b, ..., aa, ab).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnsiLower;

impl_counter_formatter! { AnsiUpper, UPPERCASE }
impl_counter_formatter! { AnsiLower, LOWERCASE }

/// Converts ANSI letters (e.g., "A", "Z", "AA") to their corresponding numeric values.
/// The conversion follows Excel's alphabetic counter rules: 'A' = 1, 'B' = 2, ...,
/// 'Z' = 26, 'AA' = 27, etc.
///
/// The `UPPERCASE` constant determines whether the string should be validated
/// as uppercase or lowercase.
///
/// # Returns
///
/// Returns `Some(u32)` if conversion is successful; otherwise, returns `None`.
#[inline]
fn convert_letters_to_number<const UPPERCASE: bool>(value: &str) -> Option<u32> {
	if value.is_empty() {
		return None;
	}

	if UPPERCASE {
		if !value.chars().all(|c| c.is_ascii_uppercase()) {
			return None;
		}
	} else if !value.chars().all(|c| c.is_ascii_lowercase()) {
		return None;
	}

	let result = value.chars().rev().enumerate().fold(0_u32, |acc, (i, c)| {
		acc + ((c as u32) - (if UPPERCASE { 'A' } else { 'a' } as u32) + 1) * 26_u32.pow(i as u32)
	});

	Some(result)
}

/// Writes the numeric value as ANSI letters (e.g., 1 → "A", 27 → "AA") into the provided buffer.
///
/// # Arguments
///
/// * `num` - The numeric value to convert.
/// * `width` - The minimum width of the generated string, padded with zeros if necessary.
/// * `buf` - The buffer to write the resulting string into.
#[inline]
fn write_number_as_letters<const UPPERCASE: bool>(
	num: u32,
	width: usize,
	buf: &mut impl fmt::Write,
) -> fmt::Result {
	let base = if UPPERCASE { b'A' } else { b'a' };
	write_number_as_letters_gen(num, width, 26, |r| (base + r as u8) as char, buf)
}
