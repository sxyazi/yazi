use std::mem;

use super::{DateField, Locale, LocaleError, State, Width};

pub(super) struct DateParser<'a> {
	locale:             &'a Locale,
	pub(super) state:   State,
	pub(super) literal: String,
	fields:             [Width; 3],
	output:             String,
}

impl<'a> DateParser<'a> {
	pub(super) fn new(locale: &'a Locale) -> Self {
		Self {
			locale,
			state: State::Pattern,
			literal: String::new(),
			fields: [Width::None; 3],
			output: String::new(),
		}
	}

	pub(super) fn push(&mut self, field: DateField, system: Width) -> Result<(), LocaleError> {
		let (width, c) = match field {
			DateField::Year => (self.locale.year, 'Y'),
			DateField::Month => (self.locale.month, 'm'),
			DateField::Day => (self.locale.day, 'd'),
		};

		if !mem::replace(&mut self.fields[field as usize], system).is_none() {
			return Err(LocaleError);
		}

		let width = width.resolve(system);
		if !width.is_none() && !self.output.is_empty() {
			self.push_separator();
		}
		width.push(c, &mut self.output);

		self.literal.clear();
		self.state = State::Pattern;
		Ok(())
	}

	fn push_separator(&mut self) {
		// `literal` contains the text between date fields in the system's pattern.
		// Use `-` if there is no separator (`yyyyMMdd`) or the text contains letters
		// or numbers (`yyyy年MM月dd日`): both become `%Y-%m-%d` when all fields are enabled.
		if self.literal.is_empty() || self.literal.chars().any(char::is_alphanumeric) {
			self.output.push('-');
			return;
		}

		// Keep punctuation and whitespace: `yyyy/MM/dd` becomes `%Y/%m/%d`.
		for c in self.literal.chars() {
			self.output.push(c);
			// Escape a literal `%` as `%%`, so formatting displays `%` instead of
			// interpreting it as a directive: `yyyy '%' MM '%' dd` becomes `%Y %% %m %% %d`.
			if c == '%' {
				self.output.push('%');
			}
		}
	}

	pub(super) fn finish(self) -> Result<String, LocaleError> {
		if self.fields.iter().any(|width| width.is_none()) { Err(LocaleError) } else { Ok(self.output) }
	}
}
