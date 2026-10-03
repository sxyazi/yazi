use std::{ffi::CStr, sync::LazyLock};

use super::{DateField, DateParser, Locale, LocaleError, State, TimeField, TimeParser, Width};

impl Locale {
	pub(super) fn date_pattern(&self) -> Result<String, LocaleError> {
		static PATTERN: LazyLock<String> = LazyLock::new(|| {
			let pat = unsafe { CStr::from_ptr(libc::nl_langinfo(libc::D_FMT)) }.to_string_lossy();
			pat.replace("%D", "%m/%d/%y").replace("%F", "%Y-%m-%d")
		});

		DateParser::new(self).parse(&PATTERN)
	}

	pub(super) fn time_widths(&self) -> Result<[Width; 3], LocaleError> {
		static WIDTHS: LazyLock<[Width; 3]> = LazyLock::new(|| {
			let pat = unsafe { CStr::from_ptr(libc::nl_langinfo(libc::T_FMT)) }.to_string_lossy();
			let am_pm = unsafe { CStr::from_ptr(libc::nl_langinfo(libc::T_FMT_AMPM)) }.to_string_lossy();
			TimeParser::new().parse(&pat, &am_pm)
		});

		Ok(*WIDTHS)
	}
}

impl DateParser<'_> {
	pub(super) fn parse(mut self, pat: &str) -> Result<String, LocaleError> {
		for c in pat.chars() {
			match (self.state, c) {
				(State::Pattern, '%') => self.state = State::Percent,
				(State::Percent, '%') => {
					self.literal.push('%');
					self.state = State::Pattern;
				}
				(State::Percent | State::Modified(_), c) if Width::is_modifier(c) => {
					self.state = State::Modified(Width::from_modifier(c, self.state));
				}
				(State::Percent | State::Modified(_), _) if let Some(field) = DateField::new(c) => {
					self.push(field, Width::from_directive(c, self.state))?;
				}
				(State::Percent | State::Modified(_), _) => {
					self.literal.clear();
					self.state = State::Pattern;
				}
				_ => self.literal.push(c),
			}
		}

		self.finish()
	}
}

impl TimeParser {
	pub(super) fn parse(mut self, pat: &str, am_pm: &str) -> [Width; 3] {
		for c in pat.chars() {
			match (self.state, c) {
				(State::Pattern, '%') => self.state = State::Percent,
				(State::Percent, '%') => self.state = State::Pattern,
				(State::Percent | State::Modified(_), c) if Width::is_modifier(c) => {
					self.state = State::Modified(Width::from_modifier(c, self.state));
				}
				(State::Percent | State::Modified(_), c) if Self::is_compound(c) => {
					let pat = Self::compound_pattern(c, am_pm);
					self.widths = Self { state: State::Pattern, widths: self.widths }.parse(pat, "");
					self.state = State::Pattern;
				}
				(State::Percent | State::Modified(_), _) if let Some(field) = TimeField::new(c) => {
					self.widths[field as usize] = Width::from_directive(c, self.state);
					self.state = State::Pattern;
				}
				(State::Percent | State::Modified(_), _) => self.state = State::Pattern,
				_ => (),
			}
		}

		self.widths
	}

	fn is_compound(c: char) -> bool { matches!(c, 'r' | 'R' | 'T') }

	fn compound_pattern(c: char, am_pm: &str) -> &str {
		match c {
			// `%r` uses the system's full 12-hour time pattern from T_FMT_AMPM,
			// typically `%I:%M:%S %p`, e.g. `03:05:07 PM`.
			'r' => am_pm,
			// `%R` means 24-hour hours and minutes, e.g. `15:05`.
			'R' => "%H:%M",
			// `%T` means 24-hour hours, minutes and seconds, e.g. `15:05:07`.
			_ => "%H:%M:%S",
		}
	}
}

impl Width {
	fn is_modifier(c: char) -> bool { matches!(c, '-' | '_' | '0'..='9' | '^' | '#' | 'E' | 'O') }

	fn from_modifier(c: char, state: State) -> Self {
		match (c, state) {
			// `-` removes padding; `_` pads with spaces. Neither uses leading zeroes,
			// so record both as Numeric: `%-m` displays `2`, while `%_m` displays ` 2`.
			('-' | '_', _) => Self::Numeric,

			// `0` requests zero-padding: `%0e` displays `05` instead of the usual ` 5`.
			('0', _) => Self::TwoDigit,

			// Other modifiers do not change the recorded padding, e.g. in `%-Om`, `O`
			// selects alternative digits, but the earlier `-` still means no padding.
			(_, State::Modified(width)) => width,

			// No padding choice has been seen yet.
			_ => Self::None,
		}
	}

	fn from_directive(c: char, state: State) -> Self {
		match (c, state) {
			// `%Y` means the full year, e.g. `2026`, regardless of padding flags from modifiers.
			('Y', _) => Self::Numeric,

			// `%y` means the last two year digits, e.g. `26`, regardless of padding flags from modifiers.
			('y', _) => Self::TwoDigit,

			// An explicit padding choice overrides the default for fields other than year:
			// `%-m` records Numeric, while `%0e` records TwoDigit.
			(_, State::Modified(width)) if !width.is_none() => width,

			// Numeric month, day, hour, minute and second directives use leading zeroes
			// by default, e.g. `%m` displays `02` and `%H` displays `09`.
			('m' | 'd' | 'H' | 'I' | 'M' | 'S', _) => Self::TwoDigit,

			// `%e`/`%k`/`%l` use space-padding, and `%b`/`%B`/`%h` are month names.
			// Record them as unpadded numbers,
			// since our output uses numeric dates rather than spaces or month names.
			_ => Self::Numeric,
		}
	}
}

impl DateField {
	pub(super) fn new(char: char) -> Option<Self> {
		match char {
			'Y' | 'y' => Some(Self::Year),
			'm' | 'b' | 'B' | 'h' => Some(Self::Month),
			'd' | 'e' => Some(Self::Day),
			_ => None,
		}
	}
}

impl TimeField {
	pub(super) fn new(c: char) -> Option<Self> {
		match c {
			'H' | 'I' | 'k' | 'l' => Some(Self::Hour),
			'M' => Some(Self::Minute),
			'S' => Some(Self::Second),
			_ => None,
		}
	}
}
