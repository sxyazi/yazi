use std::ptr;

use windows_sys::Win32::Globalization::{GetLocaleInfoEx, LOCALE_SSHORTDATE, LOCALE_STIMEFORMAT};

use super::{DateField, DateParser, Locale, LocaleError, State, TimeField, TimeParser, Width};

impl Locale {
	pub(super) fn date_pattern(&self) -> Result<String, LocaleError> {
		DateParser::new(self).parse(&Self::pattern(LOCALE_SSHORTDATE)?)
	}

	pub(super) fn time_widths(&self) -> Result<[Width; 3], LocaleError> {
		Ok(TimeParser::new().parse(&Self::pattern(LOCALE_STIMEFORMAT)?))
	}

	fn pattern(kind: u32) -> Result<String, LocaleError> {
		let mut pat = [0; 80];
		let len = unsafe { GetLocaleInfoEx(ptr::null(), kind, pat.as_mut_ptr(), pat.len() as i32) };

		if len == 0 { Err(LocaleError) } else { Ok(String::from_utf16_lossy(&pat[..len as usize - 1])) }
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

					// Windows uses `d`/`dd` for the day of the month (e.g. `5`/`05`),
					// and `ddd`/`dddd` for the weekday (e.g. `Mon`/`Monday`).
					if field.is_day() && len > 2 {
						self.literal.clear();
						continue;
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
			'M' => Some(Self::Month),
			'd' => Some(Self::Day),
			_ => None,
		}
	}
}

impl TimeField {
	pub(super) fn new(c: char) -> Option<Self> {
		match c {
			'h' | 'H' => Some(Self::Hour),
			'm' => Some(Self::Minute),
			's' => Some(Self::Second),
			_ => None,
		}
	}
}
