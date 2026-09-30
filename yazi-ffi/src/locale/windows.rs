use chrono::{DateTime, Datelike, Local, Utc};
use windows::{Foundation, Globalization::DateTimeFormatting::DateTimeFormatter};

use super::{Locale, LocaleError, Width, common::{Field, Part}};

impl Locale {
	pub fn format(&self, date: DateTime<Utc>) -> Result<Vec<u8>, LocaleError> {
		if self.has_date() && !(1601..=30827).contains(&date.with_timezone(&Local).year()) {
			return Err(LocaleError);
		}
		let ticks = date
			.timestamp()
			.checked_add(11_644_473_600)
			.and_then(|seconds| seconds.checked_mul(10_000_000))
			.ok_or(LocaleError)?;
		let time = Foundation::DateTime { UniversalTime: ticks };
		let mut output = Vec::new();
		if self.has_date() {
			let source = DateTimeFormatter::ShortDate().map_err(|_| LocaleError)?;
			output.extend(self.component(time, source).map_err(|_| LocaleError)?);
		}
		if self.has_time() {
			if !output.is_empty() {
				output.push(b' ');
			}
			let source = DateTimeFormatter::LongTime().map_err(|_| LocaleError)?;
			output.extend(self.component(time, source).map_err(|_| LocaleError)?);
		}
		Ok(output)
	}

	pub(super) fn token(self, field: Field, width: Width, original: &str, output: &mut String) {
		let name = match field {
			Field::Year if matches!(width, Width::TwoDigit) => "year.abbreviated",
			Field::Year => "year.full",
			Field::Month => "month.integer",
			Field::Day => "day.integer",
			Field::Hour => "hour.integer",
			Field::Minute => "minute.integer",
			Field::Second => "second.integer",
			Field::Meridiem => {
				output.push_str(original);
				return;
			}
			Field::Ignore => return,
		};
		output.push('{');
		output.push_str(name);
		if !matches!(field, Field::Year) {
			output.push_str(if matches!(width, Width::TwoDigit) { "(2)" } else { "(1)" });
		}
		output.push('}');
	}

	pub(super) fn literal(text: &str, output: &mut String) { output.push_str(text); }

	fn parts(pattern: &str) -> Vec<Part> {
		let mut output = Vec::new();
		for part in pattern.split_inclusive('}') {
			let Some((literal, token)) = part.split_once('{') else {
				output.push(Part { text: part.into(), field: None });
				continue;
			};
			if !literal.is_empty() {
				output.push(Part { text: literal.into(), field: None });
			}
			let field = match token.split('.').next().unwrap() {
				"year" => Some(Field::Year),
				"month" => Some(Field::Month),
				"day" => Some(Field::Day),
				"hour" => Some(Field::Hour),
				"minute" => Some(Field::Minute),
				"second" => Some(Field::Second),
				"period" => Some(Field::Meridiem),
				"openbrace}" | "closebrace}" => None,
				_ => Some(Field::Ignore),
			};
			output.push(Part { text: part[literal.len()..].into(), field });
		}
		output
	}

	fn component(
		&self,
		time: Foundation::DateTime,
		source: DateTimeFormatter,
	) -> windows::core::Result<Vec<u8>> {
		let pattern = source.Patterns()?.GetAt(0)?.to_string();
		let pattern = self.render(&Self::parts(&pattern));
		let clock = if self.hour24 { "24HourClock".into() } else { source.Clock()? };
		let formatter = DateTimeFormatter::CreateDateTimeFormatterContext(
			&pattern.into(),
			&source.Languages()?,
			&source.GeographicRegion()?,
			&source.Calendar()?,
			&clock,
		)?;
		Ok(formatter.Format(time)?.to_string().into_bytes())
	}
}
