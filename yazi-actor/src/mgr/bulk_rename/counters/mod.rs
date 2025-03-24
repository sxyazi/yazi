//! This module provides functionality for creating and managing various formats
//! of counters.
//!
//! Counters are used to generate sequences of values based on different
//! alphabets and numeral systems, including ANSI, Cyrillic, and Roman letters,
//! as well as digits.
//!
//! # Overview
//!
//! The module defines traits and structures for different formats of counters,
//! including:
//!
//! - uppercase and lowercase ANSI letters;
//! - uppercase and lowercase Cyrillic letters;
//! - numeric counter;
//! - uppercase and lowercase Roman numerals.
//!
//! The `CharacterCounter` structure provides a unified interface for handling
//! these different formats of counters.

use super::filename_template::CounterBuilder;
use std::fmt;

#[cfg(test)]
mod test;

#[macro_use]
mod geneal;
mod ansi;
mod cyrillic;
mod digit;
mod roman;

const UPPERCASE: bool = true;
const LOWERCASE: bool = false;

pub use ansi::{AnsiLower, AnsiUpper};
pub use cyrillic::{CyrillicLower, CyrillicUpper};
pub use digit::Digits;
use geneal::write_number_as_letters_gen;
pub use roman::{RomanLower, RomanUpper};

/// Defines common behavior for counters that generate sequential values.
pub trait Counter {
	/// Writes the current value to the provided buffer.
	fn write_value(&self, buf: &mut impl fmt::Write) -> fmt::Result;

	/// Advances the counter to the next value in the sequence.
	fn advance(&mut self);

	/// Resets the counter to its initial value.
	#[allow(dead_code)]
	fn restart(&mut self);
}

pub trait CounterFormatter: Copy {
	/// Formats a value as a zero-padded string and writes it to a buffer.
	///
	/// # Arguments
	///
	/// * `value` - The numeric value to format.
	/// * `width` - The minimum width of the output string.
	/// * `buf` - A mutable reference to a buffer.
	fn value_to_buffer(
		self,
		value: u32,
		width: usize,
		buf: &mut impl fmt::Write,
	) -> Result<(), fmt::Error>;

	/// Parses a zero-padded numeric string into a `u32` value.
	fn string_to_value(self, value: &str) -> Option<u32>;
}

/// Enum representing different formats of character-based counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterFormat {
	/// Numeric values (1, 2, ..., 999).
	Digits(Digits),

	/// Uppercase ANSI letters (A, B, C, ..., AA, AB, ...).
	AnsiUpper(AnsiUpper),

	/// Lowercase ANSI letters (a, b, c, ..., aa, ab, ...).
	AnsiLower(AnsiLower),

	/// Uppercase Roman numerals (I, II, III, IV, V, ...).
	RomanUpper(RomanUpper),

	/// Lowercase Roman numerals (i, ii, iii, iv, v, ...).
	RomanLower(RomanLower),

	/// Uppercase Cyrillic letters (А, Б, В, ..., АА, АБ, ...).
	CyrillicUpper(CyrillicUpper),

	/// Lowercase Cyrillic letters (а, б, в, ..., аа, аб, ...).
	CyrillicLower(CyrillicLower),
}

impl Default for CounterFormat {
	fn default() -> Self {
		CounterFormat::Digits(Digits)
	}
}

impl CounterFormatter for CounterFormat {
	fn value_to_buffer(
		self,
		value: u32,
		width: usize,
		buf: &mut impl fmt::Write,
	) -> Result<(), fmt::Error> {
		match self {
			CounterFormat::Digits(fmt) => fmt.value_to_buffer(value, width, buf),
			CounterFormat::AnsiUpper(fmt) => fmt.value_to_buffer(value, width, buf),
			CounterFormat::AnsiLower(fmt) => fmt.value_to_buffer(value, width, buf),
			CounterFormat::RomanUpper(fmt) => fmt.value_to_buffer(value, width, buf),
			CounterFormat::RomanLower(fmt) => fmt.value_to_buffer(value, width, buf),
			CounterFormat::CyrillicUpper(fmt) => fmt.value_to_buffer(value, width, buf),
			CounterFormat::CyrillicLower(fmt) => fmt.value_to_buffer(value, width, buf),
		}
	}

	fn string_to_value(self, value: &str) -> Option<u32> {
		match self {
			CounterFormat::Digits(fmt) => fmt.string_to_value(value),
			CounterFormat::AnsiUpper(fmt) => fmt.string_to_value(value),
			CounterFormat::AnsiLower(fmt) => fmt.string_to_value(value),
			CounterFormat::RomanUpper(fmt) => fmt.string_to_value(value),
			CounterFormat::RomanLower(fmt) => fmt.string_to_value(value),
			CounterFormat::CyrillicUpper(fmt) => fmt.string_to_value(value),
			CounterFormat::CyrillicLower(fmt) => fmt.string_to_value(value),
		}
	}
}

impl fmt::Display for CounterFormat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			CounterFormat::Digits(_) => write!(f, "Numeric Digits"),
			CounterFormat::AnsiUpper(_) => write!(f, "ANSI Uppercase Letters"),
			CounterFormat::AnsiLower(_) => write!(f, "ANSI Lowercase Letters"),
			CounterFormat::RomanUpper(_) => write!(f, "Roman Uppercase Numerals"),
			CounterFormat::RomanLower(_) => write!(f, "Roman Lowercase Numerals"),
			CounterFormat::CyrillicUpper(_) => write!(f, "Cyrillic Uppercase Letters"),
			CounterFormat::CyrillicLower(_) => write!(f, "Cyrillic Lowercase Letters"),
		}
	}
}

/// Represents a character-based counter. Provides a unified interface for
/// handling different formats of counters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterCounter {
	/// The format of counter (e.g., ANSI, Cyrillic, Roman, digits).
	format: CounterFormat,

	/// The initial numeric value of the counter, used to reset the counter.
	start: u32,

	/// The current numeric value of the counter.
	state: u32,

	/// The increment step size when advancing the counter.
	step: u32,

	/// The minimum width of the generated string, padded with leading zeros.
	width: usize,
}

impl CharacterCounter {
	/// Creates a new `CharacterCounter` instance.
	///
	/// # Arguments
	///
	/// * `format` - the format of counter (e.g., ANSI, Cyrillic, Roman, digits).
	/// * `start` - the initial numeric value of the counter.
	/// * `step` - the increment step size when advancing the counter.
	/// * `width` - the minimum width of the generated string, padded with leading zeros.
	pub fn new(format: CounterFormat, start: u32, step: u32, width: usize) -> Self {
		Self { format, start, state: start, step, width }
	}

	/// Updates the `CharacterCounter` instance with the parameters set in
	/// builder.
	pub fn update_from(&mut self, builder: CounterBuilder) {
		if self.format != builder.format() {
			self.format = builder.format();
		}

		if let Some(start) = builder.start() {
			self.start = start;
			self.state = start;
		}

		if let Some(step) = builder.step() {
			self.step = step;
		}

		if let Some(width) = builder.width() {
			self.width = width;
		}
	}
}

impl Counter for CharacterCounter {
	fn write_value(&self, buf: &mut impl fmt::Write) -> fmt::Result {
		self.format.value_to_buffer(self.state, self.width, buf)
	}

	fn advance(&mut self) {
		self.state += self.step;
	}

	fn restart(&mut self) {
		self.state = self.start;
	}
}
