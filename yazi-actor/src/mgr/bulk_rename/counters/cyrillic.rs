//! This module provides functionality for managing Cyrillic letter counters for both
//! uppercase and lowercase letters, following Excel's alphabetic counter style.

use super::{CounterFormatter, LOWERCASE, UPPERCASE, write_number_as_letters_gen};
use std::fmt;

/// An array of uppercase Cyrillic letters used for indexing and mapping.
/// This array includes all uppercase Cyrillic letters excluding 'Ё', 'Й', 'Ъ', 'Ы', 'Ь'.
const UPPERCASE_CYRILLIC: [char; 28] = [
	'А', 'Б', 'В', 'Г', 'Д', 'Е', 'Ж', 'З', 'И', 'К', 'Л', 'М', 'Н', 'О', 'П', 'Р', 'С', 'Т', 'У',
	'Ф', 'Х', 'Ц', 'Ч', 'Ш', 'Щ', 'Э', 'Ю', 'Я',
];

/// An array of lowercase Cyrillic letters used for indexing and mapping.
/// This array includes all lowercase Cyrillic letters excluding 'ё', 'й', 'ъ', 'ы', 'ь'.
const LOWERCASE_CYRILLIC: [char; 28] = [
	'а', 'б', 'в', 'г', 'д', 'е', 'ж', 'з', 'и', 'к', 'л', 'м', 'н', 'о', 'п', 'р', 'с', 'т', 'у',
	'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'э', 'ю', 'я',
];

/// A helper structure for generating uppercase Cyrillic letters (e.g., А, Б, В, ..., АА, АБ),
/// while excluding 'Ё', 'Й', 'Ъ', 'Ы' and 'Ь'.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CyrillicUpper;

/// A helper structure for generating lowercase Cyrillic letters (e.g., а, б, в, ..., аа, аб),
/// while excluding 'ё', 'й', 'ъ', 'ы' and 'ь'.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CyrillicLower;

impl_counter_formatter! { CyrillicUpper, UPPERCASE }
impl_counter_formatter! { CyrillicLower, LOWERCASE }

/// Converts Cyrillic letters (e.g., "Б", "В", "БА") to their corresponding numeric values.
/// The conversion follows Excel's alphabetic counter rules: 'А' = 1, 'Б' = 2, ...,
/// 'Я' = 28, 'АА' = 29, etc.
///
/// The `UPPERCASE` constant determines whether the string should be validated
/// as uppercase or lowercase.
///
/// # Returns
///
/// Returns `Some(u32)` if conversion is successful; otherwise, returns `None`.
#[inline]
fn convert_letters_to_number<const UPPERCASE: bool>(value: &str) -> Option<u32> {
	if invalid_string::<UPPERCASE>(value) {
		return None;
	}
	let lookup = if UPPERCASE { &UPPERCASE_CYRILLIC } else { &LOWERCASE_CYRILLIC };

	let result = value.chars().rev().enumerate().fold(0_u32, |acc, (i, c)| {
		if let Some(index) = lookup.iter().position(|&x| x == c) {
			acc + (index as u32 + 1) * 28_u32.pow(i as u32)
		} else {
			acc
		}
	});
	Some(result)
}

/// Writes the numeric value as Cyrillic letters (e.g., 1 → "А", 28 → "Я") into the provided buffer.
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
	let lookup = if UPPERCASE { &UPPERCASE_CYRILLIC } else { &LOWERCASE_CYRILLIC };

	write_number_as_letters_gen(num, width, 28, |remainder| lookup[remainder as usize], buf)
}

/// Checks if a string is non-empty and consists only of valid uppercase or
/// lowercase Cyrillic letters, excluding 'Ё', 'Й', 'Ъ', 'Ы', and 'Ь'
/// ('ё', 'й', 'ъ', 'ы' and 'ь').
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
	if UPPERCASE {
		!str.chars().all(|c| {
			// ('А'..='Я') == ('\u{0410}'..='\u{042F}')
			('\u{0410}'..='\u{042F}').contains(&c) && !matches!(c, 'Ё' | 'Й' | 'Ъ' | 'Ы' | 'Ь')
		})
	} else {
		!str.chars().all(|c| {
			// ('а'..='я') == ('\u{0430}'..='\u{044F}')
			('\u{0430}'..='\u{044F}').contains(&c) && !matches!(c, 'ё' | 'й' | 'ъ' | 'ы' | 'ь')
		})
	}
}
