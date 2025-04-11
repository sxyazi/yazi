//! This module provides functionality to generate file paths from input strings
//! containing counters and template variables. It parses lines of text and
//! produces either fixed filenames or filenames based on templates with
//! dynamically updated counters.

use std::{error::Error, fmt, fmt::Write, ops::Range, path::PathBuf};

use unicode_width::UnicodeWidthStr;

use super::{
	counters::Counter,
	filename_template::{ParseError, ParsedLine, TemplatePart},
};

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
/// counters in any line, the function records a
/// [`PathGenError::MismatchCounters`].
///
/// # Error Handling
///
/// Instead of stopping execution at the first encountered error, the function
/// collects all errors in a [`Vec<PathGenError>`], allowing the caller to see
/// every problematic line at once.
///
/// # Flexibility
///   
/// While the number of counters per line must be consistent, individual counter
/// parameters (such as format, start, step, and width) may vary line by line.
/// Counters update their values accordingly based on each line’s
/// specifications.
///
/// # Arguments
///
/// * `lines` - An Iterator with elements representing either a fixed filename
///   or a counter-based template.
pub fn generate_names<'a, T>(lines: &mut T) -> Result<Vec<PathBuf>, Vec<PathGenError<'a>>>
where
	T: Iterator<Item = &'a str>,
{
	let mut results = Vec::new();
	let mut errors = Vec::new();

	let mut counters = Vec::new();

	for (idx, line) in lines.enumerate() {
		match ParsedLine::try_from(line) {
			Ok(ParsedLine::Fixed(literal)) => {
				results.push(PathBuf::from(literal));
			}
			Ok(ParsedLine::Countable(template)) => {
				if counters.is_empty() {
					counters.extend(template.parts().iter().filter_map(|part| match part {
						TemplatePart::Text(_) => None,
						TemplatePart::CounterBuilder(builder) => Some(builder.build()),
					}))
				}

				if counters.len() != template.counter_count() {
					errors.push(PathGenError::MismatchCounters {
						expected: counters.len(),
						got: template.counter_count(),
						line_number: idx + 1,
						content: line,
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

				results.push(PathBuf::from(out));
			}
			Err(parse_err) => {
				errors.push(PathGenError::ParseError { line_number: idx + 1, error: parse_err });
			}
		}
	}

	if errors.is_empty() { Ok(results) } else { Err(errors) }
}

/// Represents errors that can occur during filename generation.
#[derive(Debug, PartialEq, Eq)]
pub enum PathGenError<'a> {
	/// Error parsing a line into a valid counter template.
	ParseError { line_number: usize, error: ParseError<'a> },

	/// Error indicating mismatch between the expected and actual
	/// number of counters at some line
	MismatchCounters { expected: usize, got: usize, line_number: usize, content: &'a str },
}

impl fmt::Display for PathGenError<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.write_to(f, u16::MAX)
	}
}

impl Error for PathGenError<'_> {}

impl PathGenError<'_> {
	/// Writes a formatted error message to a provided formatter or buffer, taking
	/// into account terminal width for enhanced readability.
	///
	/// This method formats error messages differently depending on the available
	/// width of the terminal, trimming and adjusting content to prevent line
	/// overflow and maintain clarity.
	///
	/// # Arguments
	///
	/// * `out` - A mutable reference to an object implementing the `Write` trait
	///   where the formatted error message will be written.
	/// * `term_width` - The maximum width of the terminal, which determines how
	///   the error message should be formatted.
	pub fn write_to(&self, out: &mut impl Write, term_width: u16) -> fmt::Result {
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
			PathGenError::ParseError { line_number, error } => {
				let Range { start, end } = error.span;

				// Calculate the width needed for the line number.
				let line_num_width = print_width(*line_number);

				// Calculate the width needed to print the line number and
				// separator (e.g., "182| ").
				let available = term_width.saturating_sub(line_num_width as u16 + 2) as usize;

				// Prepare the input string for display and calculate its full width.
				let mut print_input = error.input;
				let input_len_full = UnicodeWidthStr::width(print_input);
				// Calculate the width of the string before the error start.
				let mut input_len_left = UnicodeWidthStr::width(&print_input[..start]);
				// Calculate the width of the error span, ensuring at least 1 character.
				let input_len_span = UnicodeWidthStr::width(&print_input[start..end]).max(1);

				// Constructs a hint string indicating expected and found values, if applicable.
				let mut hint = String::new();
				let mut hint_len = 0;

				if let Some(exp) = &error.expected {
					let _ = write!(hint, " Expected: '{}'", exp);
					hint_len = UnicodeWidthStr::width(hint.as_str());
				}
				if let Some(fnd) = &error.found {
					let _ = write!(hint, ", found: '{}'", fnd);
					hint_len = UnicodeWidthStr::width(hint.as_str());
				}

				// Determines formatting style based on available width.
				let mut could_use_pretty_print = false;
				let mut prepend_dots = false;
				let mut append_dots = false;

				// Check if there's enough space to display the full input string and the error
				// details
				if available >= input_len_full && available >= input_len_left + input_len_span + hint_len {
					could_use_pretty_print = true;
				} else if available >= input_len_left + input_len_span + UnicodeWidthStr::width("…")
					&& available >= input_len_left + input_len_span + hint_len
				{
					// The full input doesn't fit, but there's space for the left part, error span,
					// ellipsis ("…") and the error details.

					// Truncate the input string to the end of the error position
					print_input = &print_input[..end];
					// Indicate that an ellipsis should be appended to show truncation at the end
					append_dots = true;
					could_use_pretty_print = true;
				} else if available
					>= UnicodeWidthStr::width("…") + UnicodeWidthStr::width(&print_input[start..])
					&& available >= UnicodeWidthStr::width("…") + input_len_span + hint_len
				{
					// There's space for an ellipsis, the error span from start, and hint
					// Truncate the input string to start at the error position
					print_input = &print_input[start..];
					// Indicate that an ellipsis should be prepended to show truncation at the start
					prepend_dots = true;

					// Reset the left length since we're starting from the error position
					input_len_left = 0;
					could_use_pretty_print = true;
				}

				// Print the error header
				write!(out, "Error: {}", error.reason)?;

				if could_use_pretty_print {
					// Write a blank line with alignment for the line number.
					writeln!(out, "\n{:>offset$}|", "", offset = line_num_width)?;

					// Write the line number and input, with optional ellipses.
					write!(out, "{}| ", line_number)?;
					if prepend_dots {
						write!(out, "…")?;
						// Adjust the input_len_left since we printed '…'
						input_len_left = UnicodeWidthStr::width("…");
					}
					write!(out, "{}", print_input)?;
					if append_dots {
						write!(out, "…")?;
					}
					// Write the caret line indicating the error span and the hint.
					writeln!(
						out,
						"\n{:>num_offset$}| {:>offset$}{:^>length$}{}\n",
						"",
						"",
						"",
						hint,
						num_offset = line_num_width,
						offset = input_len_left,
						length = input_len_span
					)
				} else {
					// Fallback: write the hint and the full input line.
					writeln!(out, ".{}", hint)?;
					writeln!(out, "{}| {}\n", line_number, error.input)
				}
			}

			PathGenError::MismatchCounters { expected, got, line_number, content } => {
				// Calculate the width needed for the line number.
				let line_num_width = print_width(*line_number);

				// Calculate the width needed to print the line number and
				// separator (e.g., "182| ").
				let available = term_width.saturating_sub(line_num_width as u16 + 2) as usize;

				// Calculate the width of the content.
				let input_len_span = UnicodeWidthStr::width(*content).max(1);

				let hint = format!(" Expected {} counters, but got {}", expected, got);
				let hint_len = UnicodeWidthStr::width(hint.as_str());

				let mut could_use_pretty_print = false;

				if available >= input_len_span + hint_len {
					could_use_pretty_print = true;
				}

				// Print the error header
				write!(out, "Error: Mismatch counter numbers")?;

				if could_use_pretty_print {
					// Write a blank line with alignment.
					writeln!(out, "\n{:>offset$}|", "", offset = line_num_width)?;

					// Write the line number and content.
					writeln!(out, "{}| {}", line_number, content)?;

					// Write the caret line spanning the entire content and the hint.
					writeln!(
						out,
						"{:>num_offset$}| {:^>length$}{}\n",
						"",
						"",
						hint,
						num_offset = line_num_width,
						length = input_len_span
					)
				} else {
					// Fallback: write the hint and the content.
					writeln!(out, ".{}", hint)?;
					writeln!(out, "{}| {}\n", line_number, content)
				}
			}
		}
	}
}
