use std::{ffi::CStr, mem};

use chrono::{DateTime, Utc};

use super::{Locale, LocaleError, Width, common::{Field, Part}};

impl Locale {
	pub fn format(&self, date: DateTime<Utc>) -> Result<Vec<u8>, LocaleError> {
		let source = match (self.has_date(), self.has_time()) {
			(true, true) => libc::D_T_FMT,
			(true, false) => libc::D_FMT,
			_ => libc::T_FMT,
		};
		let source = Self::source(source);
		let pattern = self.render(&Self::parts(&source, 0)?);
		let pattern = std::ffi::CString::new(pattern).map_err(|_| LocaleError)?;
		#[allow(clippy::useless_conversion, reason = "time_t can be 32-bit")]
		let timestamp = date.timestamp().try_into().map_err(|_| LocaleError)?;
		let mut time = unsafe { mem::zeroed() };
		if unsafe { libc::localtime_r(&timestamp, &mut time) }.is_null() {
			return Err(LocaleError);
		}

		let mut output = vec![0; 256];
		let length =
			unsafe { libc::strftime(output.as_mut_ptr().cast(), output.len(), pattern.as_ptr(), &time) };
		if length == 0 {
			return Err(LocaleError);
		}
		output.truncate(length);
		Ok(output)
	}

	pub(super) fn token(self, field: Field, width: Width, original: &str, output: &mut String) {
		let symbol = match field {
			Field::Year if matches!(width, Width::TwoDigit) => 'y',
			Field::Year => 'Y',
			Field::Month => 'm',
			Field::Day => 'd',
			Field::Hour if !self.hour24 && (original.contains('I') || original.contains('l')) => 'I',
			Field::Hour => 'H',
			Field::Minute => 'M',
			Field::Second => 'S',
			Field::Meridiem => {
				output.push_str(original);
				return;
			}
			Field::Ignore => return,
		};
		output.push('%');
		if matches!(width, Width::Numeric) && !matches!(field, Field::Year) {
			output.push('-');
		}
		output.push(symbol);
	}

	pub(super) fn literal(text: &str, output: &mut String) {
		output.push_str(&text.replace('%', "%%"));
	}

	fn source(kind: libc::nl_item) -> String {
		unsafe { CStr::from_ptr(libc::nl_langinfo(kind)) }.to_string_lossy().into_owned()
	}

	fn parts(pattern: &str, depth: u8) -> Result<Vec<Part>, LocaleError> {
		// Locale-defined composites can refer back to themselves.
		if depth == 8 {
			return Err(LocaleError);
		}
		let mut output = Vec::new();
		let mut literal = String::new();
		let mut chars = pattern.chars().peekable();

		while let Some(char) = chars.next() {
			if char != '%' {
				literal.push(char);
				continue;
			}
			if chars.next_if_eq(&'%').is_some() {
				literal.push('%');
				continue;
			}
			if !literal.is_empty() {
				output.push(Part { text: mem::take(&mut literal), field: None });
			}

			while matches!(chars.peek(), Some('-' | '_' | '0'..='9' | '^' | '#' | 'E' | 'O')) {
				chars.next();
			}
			let Some(code) = chars.next() else { break };
			let expansion = match code {
				'c' => Some(Self::source(libc::D_T_FMT)),
				'x' => Some(Self::source(libc::D_FMT)),
				'X' => Some(Self::source(libc::T_FMT)),
				'r' => {
					let pattern = Self::source(libc::T_FMT_AMPM);
					Some(if pattern.is_empty() { "%I:%M:%S %p".into() } else { pattern })
				}
				'F' => Some("%Y-%m-%d".into()),
				'D' => Some("%m/%d/%y".into()),
				'R' => Some("%H:%M".into()),
				'T' => Some("%H:%M:%S".into()),
				_ => None,
			};
			if let Some(expansion) = expansion {
				output.extend(Self::parts(&expansion, depth + 1)?);
				continue;
			}
			let field = match code {
				'Y' | 'y' => Field::Year,
				'm' | 'b' | 'B' | 'h' => Field::Month,
				'd' | 'e' => Field::Day,
				'H' | 'k' | 'I' | 'l' => Field::Hour,
				'M' => Field::Minute,
				'S' => Field::Second,
				'p' | 'P' => Field::Meridiem,
				'n' => {
					literal.push('\n');
					continue;
				}
				't' => {
					literal.push('\t');
					continue;
				}
				_ => Field::Ignore,
			};
			output.push(Part { text: format!("%{code}"), field: Some(field) });
		}
		if !literal.is_empty() {
			output.push(Part { text: literal, field: None });
		}
		Ok(output)
	}
}
