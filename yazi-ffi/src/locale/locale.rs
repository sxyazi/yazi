use chrono::{DateTime, Local, Utc};
use mlua::{BorrowedBytes, ExternalError, FromLua, Lua, Table, Value};
use yazi_shim::bytes::BytesExt;

use super::{TimeField, Width};

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
	pub fn format(&self, date: DateTime<Utc>) -> String {
		let mut pat = self.format_date();
		self.format_time(&mut pat);

		date.with_timezone(&Local).format(&pat).to_string()
	}

	fn format_date(&self) -> String {
		let fields = [
			(self.year, 'Y', Width::Numeric),
			(self.month, 'm', Width::TwoDigit),
			(self.day, 'd', Width::TwoDigit),
		];

		let mut it = fields.into_iter().filter(|(width, ..)| !width.is_none());
		match (it.next(), it.next()) {
			(None, None) => return String::new(),
			(Some((width, ..)), None) if !width.is_system() => {}
			_ if let Ok(pat) = self.date_pattern() => return pat,
			_ => {}
		}

		let mut pat = String::with_capacity(11);
		for (width, c, default) in fields.into_iter().filter(|(w, ..)| !w.is_none()) {
			width.resolve(default).push(c, &mut pat);
			pat.push('-');
		}

		pat.pop();
		pat
	}

	fn format_time(&self, pat: &mut String) {
		let fields = [
			(self.hour12, 'I', TimeField::Hour),
			(self.hour24, 'H', TimeField::Hour),
			(self.hour_am_pm, 'I', TimeField::Hour),
			(self.minute, 'M', TimeField::Minute),
			(self.second, 'S', TimeField::Second),
		];

		let system = if fields.iter().any(|(width, ..)| width.is_system()) {
			self.time_widths().unwrap_or([Width::TwoDigit; 3])
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
