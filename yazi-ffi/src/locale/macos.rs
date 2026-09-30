use chrono::{DateTime, Utc};
use objc2_foundation::{NSDate, NSDateFormatter, NSLocale, NSString};

use super::{Locale, LocaleError, Width};

impl Locale {
	pub fn format(&self, date: DateTime<Utc>) -> Result<Vec<u8>, LocaleError> {
		let locale = NSLocale::currentLocale();
		let template = NSDateFormatter::dateFormatFromTemplate_options_locale(
			&NSString::from_str(&self.skeleton()),
			0,
			Some(&locale),
		)
		.ok_or(LocaleError)?;
		let formatter = NSDateFormatter::new();
		formatter.setLocale(Some(&locale));
		formatter.setDateFormat(Some(&NSString::from_str(&self.normalize(&template.to_string()))));
		let date = NSDate::dateWithTimeIntervalSince1970(date.timestamp() as f64);
		Ok(formatter.stringFromDate(&date).to_string().into_bytes())
	}

	fn skeleton(self) -> String {
		let mut output = String::new();
		for (width, symbol) in [
			(self.year, 'y'),
			(self.month, 'M'),
			(self.day, 'd'),
			(self.hour, if self.hour24 { 'H' } else { 'j' }),
			(self.minute, 'm'),
			(self.second, 's'),
		] {
			let Some(width) = width else { continue };
			output.extend(std::iter::repeat_n(symbol, 1 + matches!(width, Width::TwoDigit) as usize));
		}
		output
	}

	fn normalize(self, pattern: &str) -> String {
		let mut output = String::new();
		let mut quoted = false;
		let mut chars = pattern.chars().peekable();

		while let Some(char) = chars.next() {
			if char == '\'' {
				output.push(char);
				quoted = !quoted;
				continue;
			}
			if quoted {
				output.push(char);
				continue;
			}
			let width = match char {
				'y' => self.year,
				'M' | 'L' => self.month,
				'd' => self.day,
				'h' | 'H' | 'K' | 'k' => self.hour,
				'm' => self.minute,
				's' => self.second,
				_ => {
					output.push(char);
					continue;
				}
			};
			while chars.next_if_eq(&char).is_some() {}
			let Some(width) = width else { continue };
			let length = match width {
				Width::Numeric if char == 'y' => 4,
				Width::Numeric => 1,
				Width::TwoDigit => 2,
			};
			output.extend(std::iter::repeat_n(char, length));
		}

		output
	}
}
