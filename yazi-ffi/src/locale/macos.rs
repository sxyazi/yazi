use objc2_foundation::{NSDateFormatter, NSDateFormatterStyle};

use super::{DateField, DateParser, Locale, LocaleError, State, TimeField, TimeParser, Width};

impl Locale {
	pub(super) fn date_pattern(&self) -> Result<String, LocaleError> {
		let formatter = NSDateFormatter::new();
		formatter.setDateStyle(NSDateFormatterStyle::ShortStyle);

		DateParser::new(self).parse(&formatter.dateFormat().to_string())
	}

	pub(super) fn time_widths(&self) -> Result<[Width; 3], LocaleError> {
		let formatter = NSDateFormatter::new();
		formatter.setTimeStyle(NSDateFormatterStyle::MediumStyle);

		Ok(TimeParser::new().parse(&formatter.dateFormat().to_string()))
	}
}

impl DateParser<'_> {
	pub(super) fn parse(mut self, pat: &str) -> Result<String, LocaleError> {
		let mut it = pat.chars().peekable();

		while let Some(c) = it.next() {
			match (self.state, c) {
				(_, '\'') if it.next_if_eq(&'\'').is_some() => self.literal.push('\''),
				(State::Pattern, '\'') => self.state = State::Quoted,
				(State::Quoted, '\'') => self.state = State::Pattern,
				(State::Pattern, _) if let Some(field) = DateField::new(c) => {
					let mut len = 1;
					while it.next_if_eq(&c).is_some() {
						len += 1;
					}

					self.push(field, Width::from(len))?;
				}
				_ => self.literal.push(c),
			}
		}

		self.finish()
	}
}

impl TimeParser {
	pub(super) fn parse(mut self, pat: &str) -> [Width; 3] {
		let mut it = pat.chars().peekable();

		while let Some(c) = it.next() {
			match (self.state, c) {
				(_, '\'') if it.next_if_eq(&'\'').is_some() => (),
				(State::Pattern, '\'') => self.state = State::Quoted,
				(State::Quoted, '\'') => self.state = State::Pattern,
				(State::Pattern, _) if let Some(field) = TimeField::new(c) => {
					let mut len = 1;
					while it.next_if_eq(&c).is_some() {
						len += 1;
					}

					self.widths[field as usize] = Width::from(len);
				}
				_ => (),
			}
		}

		self.widths
	}
}

impl DateField {
	pub(super) fn new(char: char) -> Option<Self> {
		match char {
			'y' => Some(Self::Year),
			'M' | 'L' => Some(Self::Month),
			'd' => Some(Self::Day),
			_ => None,
		}
	}
}

impl TimeField {
	pub(super) fn new(c: char) -> Option<Self> {
		match c {
			'h' | 'H' | 'K' | 'k' => Some(Self::Hour),
			'm' => Some(Self::Minute),
			's' => Some(Self::Second),
			_ => None,
		}
	}
}
