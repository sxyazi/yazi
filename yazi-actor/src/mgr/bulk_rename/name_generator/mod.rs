//! This module provides functionality to generate file paths from input strings
//! containing counters and template variables. It parses lines of text and
//! produces either fixed filenames or filenames based on templates with
//! dynamically updated counters.

use std::{error::Error, fmt, fmt::Write, ops::Range};

use unicode_width::UnicodeWidthStr;

use super::{Tuple, counters::Counter, filename_template::{ParseError, ParsedLine, TemplatePart}};

#[cfg(test)]
mod tests;

/// Generates a sequence of file paths from an input lines of text.
///
/// Each line is parsed into either a fixed filename or a template containing
/// counters. If the line contains a template, counters are created and
/// dynamically updated across lines, ensuring consistent numbering and
/// formatting.
///
/// For details on counter syntax and template parsing, see
/// [`super::filename_template::Template::parse`].
///
/// If any line contains counters, the number of counters must remain consistent
/// across all lines. If there's a mismatch in the expected and actual number of
/// counters in any line, the function returns error.
///
/// # Error Handling
///
/// Instead of stopping execution at the first encountered error, the function
/// collects all errors, allowing the caller to see every problematic line at
/// once.
///
/// # Flexibility
///   
/// While the number of counters per line must be consistent, individual counter
/// parameters (such as format, start, step, and width) may vary line by line.
/// Counters update their values accordingly based on each line’s
/// specifications.
pub fn generate_names<'a, T>(lines: &mut T) -> Result<Vec<Tuple>, NameGenerationErrors<'a>>
where
	T: Iterator<Item = &'a str>,
{
	let mut results = Vec::<Tuple>::new();
	let mut errors = Vec::new();

	let mut counters = Vec::new();

	for (idx, line) in lines.enumerate() {
		match ParsedLine::try_from(line) {
			Ok(ParsedLine::Fixed(literal)) => {
				results.push(Tuple::new(idx, literal));
			}
			Ok(ParsedLine::Countable(template)) => {
				if counters.is_empty() {
					counters.extend(template.parts().iter().filter_map(|part| match part {
						TemplatePart::Text(_) => None,
						TemplatePart::CounterBuilder(builder) => Some(builder.build()),
					}))
				}

				if counters.len() != template.counter_count() {
					errors.push(NameGenError::MismatchCounters {
						expected:    counters.len(),
						got:         template.counter_count(),
						line_number: idx + 1,
						content:     line,
					});
					continue;
				}

				let mut out = String::new();
				let mut counter_idx = 0;

				for part in template.parts() {
					match part {
						TemplatePart::Text(text) => {
							out.push_str(text);
						}
						TemplatePart::CounterBuilder(builder) => {
							let counter = &mut counters[counter_idx];
							counter.update_from(*builder);

							let _ = counter.write_value(&mut out);
							counter.advance();

							counter_idx += 1;
						}
					}
				}

				results.push(Tuple::new(idx, out));
			}
			Err(error) => {
				errors.push(NameGenError::ParseError { line_number: idx + 1, error });
			}
		}
	}

	if errors.is_empty() { Ok(results) } else { Err(NameGenerationErrors { errors }) }
}

/// Represents errors that can occur during filename generation.
#[derive(Debug, PartialEq, Eq)]
pub enum NameGenError<'a> {
	/// Error parsing a line into a valid counter template.
	ParseError { line_number: usize, error: ParseError<'a> },

	/// Error indicating mismatch between the expected and actual
	/// number of counters at some line
	MismatchCounters {
		expected:    usize,
		got:         usize,
		line_number: usize,
		content:     &'a str,
	},
}

impl fmt::Display for NameGenError<'_> {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Calculates the number of digits in a given line number (for formatting
		// alignment).
		fn print_width(mut n: usize) -> usize {
			let mut width = 1;
			while n >= 10 {
				n /= 10;
				width += 1;
			}
			width
		}

		match self {
			NameGenError::ParseError { line_number, error } => {
				let Range { start, end } = error.span;

				// Calculate the width needed for the line number.
				let line_num_width = print_width(*line_number);

				// Calculate the width needed to print the line number and
				// separator (e.g., "182| ").
				// let available = term_width.saturating_sub(line_num_width as u16 + 2) as
				// usize;

				// Calculate the width of the string before the error start.
				let input_len_left = UnicodeWidthStr::width(&error.input[..start]);
				// Calculate the width of the error span, ensuring at least 1 character.
				let input_len_span = UnicodeWidthStr::width(&error.input[start..end]).max(1);

				// Constructs a hint string indicating expected and found values, if applicable.
				let mut hint = String::new();

				if let Some(exp) = &error.expected {
					let _ = write!(hint, " Expected: '{exp}'");
				}
				if let Some(fnd) = &error.found {
					let _ = write!(hint, ", found: '{fnd}'");
				}

				// Print the error header
				write!(fmt, "Error: {}", error.reason)?;

				// Write a blank line with alignment for the line number.
				writeln!(fmt, "\n{:>offset$}|", "", offset = line_num_width)?;

				// Write the line number and input, with optional ellipses.
				write!(fmt, "{line_number}| ")?;

				write!(fmt, "{}", error.input)?;

				// Write the caret line indicating the error span and the hint.
				writeln!(
					fmt,
					"\n{:>num_offset$}| {:>offset$}{:^>length$}{}\n",
					"",
					"",
					"",
					hint,
					num_offset = line_num_width,
					offset = input_len_left,
					length = input_len_span
				)
			}

			NameGenError::MismatchCounters { expected, got, line_number, content } => {
				// Calculate the width needed for the line number.
				let line_num_width = print_width(*line_number);

				// Calculate the width of the content.
				let input_len_span = UnicodeWidthStr::width(*content).max(1);

				let hint = format!(" Expected {expected} counters, but got {got}");

				// Print the error header
				write!(fmt, "Error: Mismatch counter numbers")?;

				// Write a blank line with alignment.
				writeln!(fmt, "\n{:>offset$}|", "", offset = line_num_width)?;

				// Write the line number and content.
				writeln!(fmt, "{line_number}| {content}")?;

				// Write the caret line spanning the entire content and the hint.
				writeln!(
					fmt,
					"{:>num_offset$}| {:^>length$}{}\n",
					"",
					"",
					hint,
					num_offset = line_num_width,
					length = input_len_span
				)
			}
		}
	}
}

impl Error for NameGenError<'_> {}

/// Represents a collection of errors that occurred during filename generation.
#[derive(Debug, PartialEq, Eq)]
pub struct NameGenerationErrors<'a> {
	pub errors: Vec<NameGenError<'a>>,
}

impl<'a> fmt::Display for NameGenerationErrors<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.errors.iter().try_for_each(|e| write!(f, "{}", e))
	}
}

impl<'a> Error for NameGenerationErrors<'a> {}
