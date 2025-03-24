use std::{
	error::Error,
	fmt,
	num::{IntErrorKind, NonZero, ParseIntError},
	ops::Range,
	str::FromStr,
	str::Split,
};
use unicode_width::UnicodeWidthStr;

use super::counters::{
	AnsiLower, AnsiUpper, CharacterCounter, CounterFormat, CounterFormatter, CyrillicLower,
	CyrillicUpper, Digits, RomanLower, RomanUpper,
};

#[cfg(test)]
mod test;
#[cfg(test)]
use super::counters::Counter;

/// A byte range within a string.
pub type Span = Range<usize>;

impl<'a> TryFrom<&'a str> for ParsedLine<'a> {
	type Error = ParseError<'a>;

	fn try_from(input: &'a str) -> Result<Self, Self::Error> {
		match Template::parse(input) {
			Ok(template) => Ok(ParsedLine::Countable(template)),
			Err(error) => match error {
				TemplateError::NotCounter => Ok(ParsedLine::Fixed(input)),
				TemplateError::Parse(error) => Err(error),
			},
		}
	}
}

/// A parsed input line representing either plain text or a template
/// combining static text fragments and counters.
pub enum ParsedLine<'a> {
	/// A variant used when the input line contains no counters.
	///
	/// This avoids allocating a `Vec` for `TemplatePart` when no dynamic parts are present.
	Fixed(&'a str),

	/// A variant used when the input line contains a template with
	/// static text fragments and counters.
	Countable(Template<'a>),
}

/// A template (pattern) that combines static text fragments and counters.
///
/// This structure holds a list of parts, where each part is either plain text
/// or a placeholder for a dynamically generated counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template<'a> {
	/// A sequence of parts, each of which is either static text or a `CounterBuilder`.
	parts: Vec<TemplatePart<'a>>,

	/// Number of `CounterBuilder` parts contained in `parts`.
	///
	/// Guaranteed to be non-zero for type safety: if no counters are found,
	/// we return `ParsedLine::Fixed` instead of creating a `Template`.
	counter_count: NonZero<usize>,
}

/// Represents the elements of a countable template (pattern).
///
/// A `TemplatePart` can be either static text or a `CounterBuilder`, which
/// allows for the deferred creation or updating of a counter when needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplatePart<'a> {
	/// Static text within the template.
	Text(&'a str),

	/// A `CounterBuilder` that can be used to create or modify a counter.
	CounterBuilder(CounterBuilder),
}

impl Template<'_> {
	/// Returns the number of CounterBuilders within [`Template`]
	pub fn counter_count(&self) -> usize {
		self.counter_count.get()
	}

	/// Returns the number of CounterBuilders within [`Template`]
	pub fn parts(&self) -> &Vec<TemplatePart<'_>> {
		&self.parts
	}

	/// Parses a string into a `Template`, identifying static text and counter
	/// placeholders.
	///
	/// Parses the input text, replacing countable elements with `CounterBuilder`
	/// instances. Returns `Err(TemplateError)` if no counters are found or
	/// parsing fails.
	///
	/// # Counter Pattern Format
	///
	/// Format: `%{<COUNT_TYPE>,<START_VALUE>,<COUNT_STEP>,<COUNT_WIDTH>}`
	///
	/// * `<COUNT_TYPE>`: A single character indicating the counter type:
	///
	///     - `N`, `n`, `D`, `d` → Numeric digits.
	///     - `A` → Uppercase ANSI letters.
	///     - `a` → Lowercase ANSI letters.
	///     - `R` → Uppercase Roman numerals.
	///     - `r` → Lowercase Roman numerals.
	///     - `C` → Uppercase Cyrillic letters.
	///     - `c` → Lowercase Cyrillic letters.
	///
	/// * `<START_VALUE>` (optional): Initial value, either as a number (e.g.,
	///   `1`, `2`, etc.) or a value corresponding to the counter type:
	///
	///     - `A`, `B`, `AA` (ANSI uppercase)
	///     - `a`, `b`, `aa` (ANSI lowercase)
	///     - `I`, `II`, `III` (Roman uppercase)
	///     - `i`, `ii`, `iii` (Roman lowercase)
	///     - `А`, `Б`, `АБ` (Cyrillic uppercase)
	///     - `а`, `б`, `аб` (Cyrillic lowercase)
	///     - `_` for unspecified.
	///
	/// - `<COUNT_STEP>` (optional): Step size, integer (e.g., `2`) or `_` for
	///   unspecified.
	///
	/// - `<COUNT_WIDTH>` (optional): Minimum width with zero-padding. Integer
	///   (e.g., `3`) or `_` for unspecified.
	///
	/// Optional parameters must be specified sequentially: `<START_VALUE>` is
	/// required if `<COUNT_STEP>` or `<COUNT_WIDTH>` are used, either explicitly
	/// (e.g., `1`) or with `_`. Omitting earlier parameters with commas (e.g.,
	/// `%{N,,2}` or `%{N,,,4}`) is invalid. Use `%{N,_,2}` or `%{N,_,_,4}`
	/// instead. Defaults (`1` for unset values) apply in
	/// `CounterBuilder::build()`.
	///
	/// ## Escaping `%{`
	///
	/// To include a literal `%{` in the output without interpreting it as a counter,
	/// escape it by writing `%%{`. The leading `%%` will be interpreted as a single
	/// `%` followed by a literal `{`. For example:
	///
	/// - `file_%%{name}` → `file_%{name}`
	///
	/// # Examples (given as CounterBuilder fields)
	///
	/// - `%{N,1}`     → start=1,    step=None, width=None
	/// - `%{N,1,3}`   → start=1,    step=3,    width=None
	/// - `%{N,_,2}`   → start=None, step=2,    width=None
	/// - `%{N,_,_,4}` → start=None, step=None, width=4
	///
	/// # Errors
	///
	/// - `TemplateError::NotCounter`: No counters found.
	/// - `TemplateError::Parse`: Invalid parameters (e.g., `%{N,,2}`).
	fn parse(input: &str) -> Result<Template<'_>, TemplateError<'_>> {
		let mut chars = input.char_indices();
		let mut parts = Vec::new();
		let mut parsed_start_byte_idx = 0;
		let mut counter_count = 0;

		while let Some((count_start_byte_idx, char)) = chars.next() {
			if char == '%' {
				let mut percent_count = 1;
				while let Some((current_byte_idx, char)) = chars.next() {
					match char {
						'%' => percent_count += 1,
						'{' => {
							// If we have sequence of percent sign (`%%{` or `%%%{` or `%%%%{` and so on)
							// so there is escaping of `%{`
							if percent_count > 1 {
								// current_byte_idx points to the start of '{', so we need to subtract 2 for point
								// to the star '%%{'
								if current_byte_idx - parsed_start_byte_idx > 0 {
									let before_match = &input[parsed_start_byte_idx..current_byte_idx - 2];
									parts.push(TemplatePart::Text(before_match));
								}

								parts.push(TemplatePart::Text("%{"));

								// current_byte_idx points to the start of '{', so we need to add the length of '{'
								parsed_start_byte_idx = current_byte_idx + 1;
								break;
							} else {
								let mut counter_end_found = false;
								for (count_end_byte_idx, char) in chars.by_ref() {
									if char == '}' {
										counter_end_found = true;

										// counter starts from `count_start_byte_idx + length of %{` till `count_end_byte_idx`
										let span = count_start_byte_idx + 2..count_end_byte_idx;
										let builder = Self::parse_counter(span, input)?;

										if count_start_byte_idx - parsed_start_byte_idx > 0 {
											let before_match = &input[parsed_start_byte_idx..count_start_byte_idx];
											parts.push(TemplatePart::Text(before_match));
										}
										parts.push(TemplatePart::CounterBuilder(builder));
										parsed_start_byte_idx = count_end_byte_idx + 1;
										counter_count += 1;
										break;
									}
								}
								if !counter_end_found {
									return Err(TemplateError::Parse(ParseError {
										input,
										span: input.len()..input.len(),
										reason: "Unclosed delimiter",
										expected: Some("}"),
										found: None,
									}));
								}
								break;
							}
						}
						_ => break,
					}
				}
			}
		}

		// If no countable elements were found, return an error.
		if counter_count == 0 {
			Err(TemplateError::NotCounter)
		} else {
			// Add any remaining text after the last match.
			if parsed_start_byte_idx < input.len() {
				let after_last_match = &input[parsed_start_byte_idx..];
				parts.push(TemplatePart::Text(after_last_match));
			}
			Ok(Template {
				parts,
				// SAFETY: We checked that counter_count is not equal to zero
				counter_count: unsafe { NonZero::<usize>::new_unchecked(counter_count) },
			})
		}
	}

	/// Parses a counter parameters from a substring and creates a `CounterBuilder`.
	///
	/// Takes a `span` range within the `input` string, extracts a counter parameters in the format
	/// `[<COUNT_TYPE>,<START_VALUE>,<COUNT_STEP>,<COUNT_WIDTH>]`, and returns a configured
	/// `CounterBuilder`. Returns `ParseError` if parsing fails.
	///
	/// See `Template::parse` for more information
	///
	/// # Arguments
	///
	/// * `span` - The range of the counter parameters within `input`.
	/// * `input` - The full input string.
	fn parse_counter(span: Span, input: &str) -> Result<CounterBuilder, ParseError<'_>> {
		let Range { start, end } = span;
		let mut iter = Parts::new(span, input);
		let (format_span, format) = iter.next().unwrap_or((start..end, ""));

		let builder = CounterBuilder::default()
			.try_set_format(input, format_span, format)?
			.try_set_start(input, iter.next())?
			.try_set_step(input, iter.next())?
			.try_set_width(input, iter.next())?;

		if let Some((format_span, _)) = iter.next() {
			return Err(ParseError {
				input,
				span: format_span.start - 1..end,
				reason: "Extra arguments",
				expected: Some("no additional arguments"),
				found: None,
			});
		}

		Ok(builder)
	}
}

/// Enum representing errors that can occur when parsing countable
/// [Template].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateError<'a> {
	/// Error indicating that no counter was found in the pattern.
	NotCounter,
	/// Error indicating invalid input for a counter configuration.
	Parse(ParseError<'a>),
}

impl<'a> From<ParseError<'a>> for TemplateError<'a> {
	fn from(err: ParseError<'a>) -> Self {
		TemplateError::Parse(err)
	}
}

/// Represents an error encountered while parsing a counter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError<'a> {
	/// The full input string where the error occurred.
	pub input: &'a str,

	/// The byte range in the input text where the error occurred.
	pub span: Span,

	/// A brief description of what went wrong.
	pub reason: &'static str,

	/// An optional hint about the expected input.
	pub expected: Option<&'static str>,

	/// An optional string showing what was actually found.
	pub found: Option<&'a str>,
}

impl fmt::Display for ParseError<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Range { start, end } = self.span;
		write!(f, "Error: {}", self.reason)?;

		if let Some(expected) = &self.expected {
			write!(f, ". Expected: '{expected}'")?;
		}
		if let Some(found) = &self.found {
			write!(f, ", found: '{found}'")?;
		}

		writeln!(f, "\n\n{}", self.input)?;

		let offset = UnicodeWidthStr::width(&self.input[..start]);
		let length = UnicodeWidthStr::width(&self.input[start..end]).max(1);

		writeln!(f, "{:>offset$}{:^>length$}", "", "", offset = offset, length = length)
	}
}

impl Error for ParseError<'_> {}

/// An iterator over comma-separated segments of a string slice, returning
/// both the segment and its byte range (`Span`) relative to the original full
/// input string.
///
/// This is used for parsing parameter lists such as `%{N,_,2,3}` where each
/// value needs to be associated with its exact location in the original string
/// for precise error reporting.
pub struct Parts<'a> {
	parts: Split<'a, char>,
	current_idx: usize,
}

impl<'a> Parts<'a> {
	/// Creates a new `Parts` iterator over the portion of `input` defined by `span`.
	///
	/// The `span` defines the byte range into the original string, and the
	/// returned segments will report their positions relative to that original input.
	pub fn new(span: Span, input: &'a str) -> Self {
		let current_idx = span.start;
		let parts = &input[span];

		Self { parts: parts.split(','), current_idx }
	}
}

impl<'a> Iterator for Parts<'a> {
	/// An item representing a single comma-separated segment and its byte range.
	///
	/// - `Span`: the byte range of the segment in the original input string,
	///   used for precise error reporting.
	///
	/// - `&'a str`: the actual content of the segment.
	type Item = (Span, &'a str);

	fn next(&mut self) -> Option<Self::Item> {
		let part = self.parts.next()?;

		let next = Some((self.current_idx..self.current_idx + part.len(), part));
		self.current_idx += part.len() + 1;
		next
	}
}

/// A builder for constructing `CharacterCounter` instances.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CounterBuilder {
	/// The format of counter to create.
	format: CounterFormat,

	/// The initial counter value.
	start: Option<u32>,

	/// The step size for incrementing the counter.
	step: Option<u32>,

	/// The minimum output width, zero-padded if needed.
	width: Option<usize>,
}

impl CounterBuilder {
	/// Creates a new `CounterBuilder` instance with default values
	#[inline]
	#[allow(dead_code)]
	pub fn new() -> Self {
		CounterBuilder::default()
	}

	/// Returns the selected counter format.
	#[inline]
	pub fn format(&self) -> CounterFormat {
		self.format
	}

	/// Returns the initial value of the counter.
	#[inline]
	pub fn start(&self) -> Option<u32> {
		self.start
	}

	/// Returns the step size for advancing the counter.
	#[inline]
	pub fn step(&self) -> Option<u32> {
		self.step
	}

	/// Returns the minimum width of the generated output.
	#[inline]
	pub fn width(&self) -> Option<usize> {
		self.width
	}

	/// Parses and sets the counter format from a single-character string.
	///
	/// This method extracts and validates a counter format from a potentially
	/// whitespace-padded string. If parsing fails, it returns a [`ParseError`]
	/// with the span adjusted to exclude leading and trailing whitespace.
	///
	/// # Supported Characters
	///
	/// - `'D'`, `'d'`, `'N'`, `'n'` → [`Digits`]
	/// - `'A'` → [`AnsiUpper`]
	/// - `'a'` → [`AnsiLower`]
	/// - `'R'` → [`RomanUpper`]
	/// - `'r'` → [`RomanLower`]
	/// - `'C'` → [`CyrillicUpper`]
	/// - `'c'` → [`CyrillicLower`]
	///
	/// # Arguments
	///
	/// * `input` - The full input string
	///
	/// * `span` - The range of the counter format within `input`.
	///
	/// * `format` - A single-character string (possibly with surrounding whitespaces)
	///   representing the counter format.
	#[inline]
	pub fn try_set_format<'a>(
		mut self,
		input: &'a str,
		span: Span,
		format: &'a str,
	) -> Result<Self, ParseError<'a>> {
		let Range { start, end } = span;
		let (trim_span, trimmed) = trim_with_range(start..end, format);
		let format = match trimmed {
			"D" | "d" | "N" | "n" => CounterFormat::Digits(Digits),
			"A" => CounterFormat::AnsiUpper(AnsiUpper),
			"a" => CounterFormat::AnsiLower(AnsiLower),
			"R" => CounterFormat::RomanUpper(RomanUpper),
			"r" => CounterFormat::RomanLower(RomanLower),
			"C" => CounterFormat::CyrillicUpper(CyrillicUpper),
			"c" => CounterFormat::CyrillicLower(CyrillicLower),
			"" => {
				return Err(ParseError {
					input,
					span: start..end,
					reason: "Empty counter kind",
					expected: Some("one of D, d, N, n, A, a, R, r, C, c"),
					found: None,
				});
			}
			other => {
				return Err(ParseError {
					input,
					span: trim_span,
					reason: "Unexpected counter kind",
					expected: Some("one of D, d, N, n, A, a, R, r, C, c"),
					found: Some(other),
				});
			}
		};

		self.format = format;
		Ok(self)
	}

	/// Parses and sets the start value for the counter from an optional,
	/// possibly whitespace-padded string.
	///
	/// The value is first interpreted using the current counter format
	/// (e.g. digits, letters, Roman numerals). If that fails, it is parsed
	/// as an integer. An underscore (`_`) means "unspecified" and is ignored.
	///
	/// Leading and trailing whitespace is excluded from the error span if parsing fails.
	///
	/// # Supported Formats
	///
	/// - Digits: `1`, `2`, `100`
	/// - ANSI Uppercase: `A`, `B`, `AA`
	/// - ANSI Lowercase: `a`, `b`, `aa`
	/// - Roman Uppercase: `I`, `II`, `III`
	/// - Roman Lowercase: `i`, `ii`, `iii`
	/// - Cyrillic Uppercase: `А`, `Б`, `АБ`
	/// - Cyrillic Lowercase: `а`, `б`, `аб`
	///
	/// # Arguments
	///
	/// * `input` - The full input string
	///
	/// * `start` – Optional `(Span, &str)` pair representing the value and
	///   its position in the original input.
	#[inline]
	pub fn try_set_start<'a>(
		mut self,
		input: &'a str,
		start: Option<(Span, &'a str)>,
	) -> Result<Self, ParseError<'a>> {
		self.start = Self::parse_field(input, start, |trimmed| self.format.string_to_value(trimmed))?;
		Ok(self)
	}

	/// Parses and sets the step size for the counter from an optional,
	/// possibly whitespace-padded string.
	///
	/// The value is parsed as an integer. An underscore (`_`) means
	/// "unspecified" and is ignored.
	///
	/// Leading and trailing whitespace is excluded from the error span if parsing fails.
	///
	/// # Arguments
	///
	/// * `input` - The full input string
	///
	/// * `step` – Optional `(Span, &str)` pair representing the value and
	///   its position in the original input.
	#[inline]
	pub fn try_set_step<'a>(
		mut self,
		input: &'a str,
		step: Option<(Span, &'a str)>,
	) -> Result<Self, ParseError<'a>> {
		self.step = Self::parse_field(input, step, |_| None)?;
		Ok(self)
	}

	/// Parses and sets the minimum output width for the counter from an optional,
	/// possibly whitespace-padded string.
	///
	/// The value is parsed as an integer. An underscore (`_`) means
	/// "unspecified" and is ignored.
	///
	/// Leading and trailing whitespace is excluded from the error span if parsing fails.
	///
	/// # Arguments
	///
	/// * `input` - The full input string
	///
	/// * `width` – Optional `(Span, &str)` pair representing the value and
	///   its position in the original input.
	#[inline]
	pub fn try_set_width<'a>(
		mut self,
		input: &'a str,
		width: Option<(Span, &'a str)>,
	) -> Result<Self, ParseError<'a>> {
		self.width = Self::parse_field(input, width, |_| None)?;
		Ok(self)
	}

	/// Parses an optional `(Span, &str)` into a typed value with optional custom logic.
	///
	/// Steps:
	///
	/// 1. Trim leading/trailing whitespace.
	/// 2. If the result is `_`, return `Ok(None)`.
	/// 3. If `custom_parse` returns `Some(val)`, use it.
	/// 4. Otherwise, parse using [`FromStr`] for `T`.
	///
	/// On failure, returns a [`ParseError`] with a span pointing to the trimmed region
	/// (or the full span if the string is empty).
	///
	/// # Type Parameters
	///
	/// - `T`: The target type, requiring [`FromStr`], [`IntError`], and [`Error`].
	///
	/// # Arguments
	///
	/// - `input`: The full input string
	/// - `field_data`: Optional `(Span, &str)` (position in the original input text and
	///   the content).
	/// - `custom_parse`: A fallback parser tried before numeric parsing.
	#[inline]
	fn parse_field<'a, T>(
		input: &'a str,
		field_data: Option<(Span, &'a str)>,
		custom_parse: impl FnOnce(&str) -> Option<T>,
	) -> Result<Option<T>, ParseError<'a>>
	where
		T: FromStr,
		T::Err: IntError + Error,
	{
		let Some((original_span, content)) = field_data else {
			return Ok(None);
		};

		let (mut trim_span, trimmed) = trim_with_range(original_span.clone(), content);

		if trimmed == "_" {
			return Ok(None);
		}

		if let Some(val) = custom_parse(trimmed) {
			return Ok(Some(val));
		}

		match trimmed.parse::<T>() {
			Ok(v) => Ok(Some(v)),
			Err(err) => {
				let reason = match err.kind() {
					IntErrorKind::Empty => {
						trim_span = original_span;
						"Cannot parse integer from empty string"
					}
					IntErrorKind::InvalidDigit => "Invalid digit found in string",
					IntErrorKind::PosOverflow => "Number too large",
					IntErrorKind::NegOverflow => "Number too small",
					_ => "Failed to parse integer",
				};
				Err(ParseError {
					input,
					span: trim_span,
					reason,
					expected: Some("digit"),
					found: Some(trimmed),
				})
			}
		}
	}

	/// Builds and returns a `CharacterCounter` instance based on the parameters
	/// set in this builder.
	///
	/// If any of these parameters are not set, default values are used:
	/// - `start`: 1
	/// - `step`: 1
	/// - `width`: 1
	#[inline]
	pub fn build(self) -> CharacterCounter {
		CharacterCounter::new(
			self.format,
			self.start.unwrap_or(1),
			self.step.unwrap_or(1),
			self.width.unwrap_or(1),
		)
	}
}

/// Trims leading whitespace from `str` and returns the trimmed slice
/// along with an updated `Span` reflecting the new start position.
///
/// If the string is entirely whitespace, returns an empty slice
/// and a zero-length span at the original `span.start`.
///
/// # Arguments
///
/// - `span`: The original byte range in the input text.
/// - `str`: The substring to trim.
fn trim_with_range(span: Span, str: &str) -> (Span, &str) {
	let Some(mut start) = str.find(|c: char| !c.is_whitespace()) else {
		return (span.start..span.start, "");
	};

	start += span.start;
	let str = str.trim();

	(start..start + str.len(), str)
}

/// A lightweight trait for accessing [`IntErrorKind`] in generic
/// number-parsing logic, without depending directly on `ParseIntError`.
trait IntError {
	/// Returns the specific kind of integer parse error.
	fn kind(&self) -> &IntErrorKind;
}

impl IntError for ParseIntError {
	#[inline]
	fn kind(&self) -> &IntErrorKind {
		self.kind()
	}
}
