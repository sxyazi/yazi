use chrono::{DateTime, Local, Utc};
use mlua::{BorrowedBytes, ExternalError, FromLua, Lua, Table, Value};
use yazi_shim::bytes::BytesExt;

use super::{LocaleError, TimeField, Width};

#[derive(Clone, Copy, Default)]
pub struct Locale {
	pub(super) year:       Width,
	pub(super) month:      Width,
	pub(super) day:        Width,
	pub(super) hour12:     Width,
	pub(super) hour24:     Width,
	pub(super) hour_am_pm: Width,
	pub(super) minute:     Width,
	pub(super) second:     Width,
}

impl Locale {
	pub fn format(&self, date: DateTime<Utc>) -> Result<String, LocaleError> {
		let mut pat = self.format_date()?;
		self.format_time(&mut pat)?;

		let mut output = String::new();
		date.with_timezone(&Local).format(&pat).write_to(&mut output)?;
		Ok(output)
	}

	fn format_date(&self) -> Result<String, LocaleError> {
		let mut it = [(self.year, 'Y'), (self.month, 'm'), (self.day, 'd')]
			.into_iter()
			.filter(|(width, _)| !width.is_none());

		let mut pat = String::new();
		match (it.next(), it.next()) {
			(None, _) => (),
			(Some((width, field)), None) if !width.is_system() => width.push(field, &mut pat),
			_ => pat = self.date_pattern()?,
		}
		Ok(pat)
	}

	fn format_time(&self, pat: &mut String) -> Result<(), LocaleError> {
		let fields = [
			(self.hour12, 'I', TimeField::Hour),
			(self.hour24, 'H', TimeField::Hour),
			(self.hour_am_pm, 'I', TimeField::Hour),
			(self.minute, 'M', TimeField::Minute),
			(self.second, 'S', TimeField::Second),
		];

		let system = if fields.iter().any(|(width, ..)| width.is_system()) {
			self.time_widths()?
		} else {
			[Width::Numeric; 3]
		};

		pat.reserve(24);
		if !pat.is_empty() {
			pat.push(' ');
		}

		for (width, c, field) in fields {
			if !width.is_none() {
				width.resolve(system[field as usize]).push(c, pat);
				pat.push(':');
			}
		}

		pat.pop();
		if !self.hour_am_pm.is_none() {
			pat.push_str(" %p");
		}

		Ok(())
	}
}

impl FromLua for Locale {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;
		let mut me = Self::default();

		for pair in t.pairs::<BorrowedBytes, Width>() {
			let (key, width) = pair?;
			match &*key {
				b"year" => me.year = width,
				b"month" => me.month = width,
				b"day" => me.day = width,
				b"hour12" => me.hour12 = width,
				b"hour24" => me.hour24 = width,
				b"hour_am_pm" => me.hour_am_pm = width,
				b"minute" => me.minute = width,
				b"second" => me.second = width,
				key => Err(format!("invalid date format field: {}", key.display()).into_lua_err())?,
			}
		}

		Ok(me)
	}
}
