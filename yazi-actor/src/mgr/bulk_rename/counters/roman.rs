//! This module provides functionality for managing Roman numeral counters
//! for both uppercase and lowercase Roman numerals.

use super::{CounterFormatter, LOWERCASE, UPPERCASE};
use std::fmt;

/// A lookup table for uppercase Roman numerals and their values.
const UPPERCASE_ROMAN_NUMERALS: [(&str, u32); 13] = [
	("M", 1000),
	("CM", 900),
	("D", 500),
	("CD", 400),
	("C", 100),
	("XC", 90),
	("L", 50),
	("XL", 40),
	("X", 10),
	("IX", 9),
	("V", 5),
	("IV", 4),
	("I", 1),
];

/// A lookup table for lowercase Roman numerals and their values.
const LOWERCASE_ROMAN_NUMERALS: [(&str, u32); 13] = [
	("m", 1000),
	("cm", 900),
	("d", 500),
	("cd", 400),
	("c", 100),
	("xc", 90),
	("l", 50),
	("xl", 40),
	("x", 10),
	("ix", 9),
	("v", 5),
	("iv", 4),
	("i", 1),
];

/// A helper structure for generating uppercase Roman numerals (e.g., I, II, III, IV, V, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomanUpper;

/// A helper structure for generating lowercase Roman numerals (e.g., i, ii, iii, iv, v, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomanLower;

impl_counter_formatter! { RomanUpper, UPPERCASE }
impl_counter_formatter! { RomanLower, LOWERCASE }

/// Converts Roman numerals (e.g. I, II, III) to their corresponding numeric values.
///
/// The `UPPERCASE` constant determines whether the string should be validated
/// as uppercase or lowercase.
///
/// # Returns
///
/// Returns `Some(u32)` if conversion is successful; otherwise, returns `None`.
#[inline]
fn convert_letters_to_number<const UPPERCASE: bool>(start: &str) -> Option<u32> {
	if invalid_string::<UPPERCASE>(start) {
		return None;
	};
	let roman_numerals =
		if UPPERCASE { &UPPERCASE_ROMAN_NUMERALS } else { &LOWERCASE_ROMAN_NUMERALS };

	let mut num = 0;
	let mut i = 0;

	while i < start.len() {
		if i + 1 < start.len() {
			if let Some(&(_, value)) = roman_numerals.iter().find(|&&(s, _)| s == &start[i..=i + 1]) {
				num += value;
				i += 2;
				continue;
			}
		}
		if let Some(&(_, value)) = roman_numerals.iter().find(|&&(s, _)| s == &start[i..=i]) {
			num += value;
			i += 1;
		} else {
			return None;
		}
	}

	Some(num)
}

/// Writes the numeric value as Roman numerals (e.g., 1 → "I", 4 → "IV") into the
/// provided buffer.
///
/// # Arguments
///
/// * `num` - The numeric value to convert.
/// * `width` - The minimum width of the generated string, padded with zeros if necessary.
/// * `buf` - The buffer to write the resulting string into.
#[inline]
fn write_number_as_letters<const UPPERCASE: bool>(
	mut num: u32,
	width: usize,
	buf: &mut impl fmt::Write,
) -> fmt::Result {
	if num == 0 {
		return Ok(());
	}

	let roman_numerals =
		if UPPERCASE { &UPPERCASE_ROMAN_NUMERALS } else { &LOWERCASE_ROMAN_NUMERALS };

	let mut stack_buf = ['0'; 10];
	let mut length = 0;

	let mut iter = roman_numerals.iter().peekable();
	'outer: while let Some(&&(roman, value)) = iter.peek() {
		'inner: loop {
			if num < value {
				break 'inner;
			}
			let final_length = length + roman.len();
			if final_length > stack_buf.len() {
				break 'outer;
			}
			for (char_ref, char) in stack_buf[length..final_length].iter_mut().zip(roman.chars()) {
				*char_ref = char
			}
			num -= value;
			length += roman.len();
		}
		iter.next();
	}

	if num > 0 {
		let mut vec_buf = Vec::with_capacity(20);
		vec_buf.extend_from_slice(&stack_buf[..length]);

		for &(roman, value) in iter {
			while num >= value {
				vec_buf.extend(roman.chars());
				num -= value;
				length += roman.len();
			}
		}

		for _ in vec_buf.len()..width {
			buf.write_char('0')?;
		}
		for &c in vec_buf.iter() {
			buf.write_char(c)?;
		}
	} else {
		for _ in length..width {
			buf.write_char('0')?;
		}
		for &c in stack_buf[..length].iter() {
			buf.write_char(c)?;
		}
	}

	Ok(())
}

/// Checks if a string is non-empty and consists only of valid
/// uppercase or lowercase Roman numerals.
///
/// The `UPPERCASE` constant determines whether to check uppercase or lowercase letters.
///
/// # Returns
///
/// Returns `true` if the string is invalid; otherwise, returns `false`.
#[inline]
fn invalid_string<const UPPERCASE: bool>(str: &str) -> bool {
	if str.is_empty() {
		return true;
	}
	let valid_chars = if UPPERCASE {
		['M', 'D', 'C', 'L', 'X', 'V', 'I']
	} else {
		['m', 'd', 'c', 'l', 'x', 'v', 'i']
	};

	!str.chars().all(|c| valid_chars.contains(&c))
}
