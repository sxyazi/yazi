use std::{ops::{Add, Deref, Sub}, str::FromStr};

use chrono::{DateTime, Utc};
use mlua::{AnyUserData, ExternalError, ExternalResult, FromLua, IntoLua, Lua, LuaString, MetaMethod, Table, UserData, UserDataFields, UserDataMethods, UserDataRef, Value};
use yazi_ffi::locale::{Locale, Width};
use yazi_shim::log::LOG_LEVEL;

use super::{DateError, Duration};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Date(DateTime<Utc>);

impl Deref for Date {
	type Target = DateTime<Utc>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl TryFrom<i64> for Date {
	type Error = DateError;

	fn try_from(secs: i64) -> Result<Self, Self::Error> {
		DateTime::from_timestamp_secs(secs).map(Self).ok_or(DateError::OutOfRange)
	}
}

impl TryFrom<f64> for Date {
	type Error = DateError;

	fn try_from(secs: f64) -> Result<Self, Self::Error> {
		if !secs.is_finite() {
			return Err(DateError::NonFinite);
		}

		let whole = secs.floor();
		let nanos = ((secs - whole) * 1_000_000_000.0).round();
		let (whole, nanos) = if nanos >= 1_000_000_000.0 { (whole + 1.0, 0.0) } else { (whole, nanos) };

		DateTime::from_timestamp(whole as i64, nanos as u32).map(Self).ok_or(DateError::OutOfRange)
	}
}

impl From<Date> for f64 {
	fn from(date: Date) -> Self {
		date.timestamp() as Self + date.timestamp_subsec_nanos() as Self / 1_000_000_000.0
	}
}

impl FromStr for Date {
	type Err = chrono::ParseError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		DateTime::parse_from_rfc3339(value).map(|value| Self(value.to_utc()))
	}
}

impl Add<f64> for Date {
	type Output = Result<Self, DateError>;

	fn add(self, secs: f64) -> Self::Output { Self::try_from(f64::from(self) + secs) }
}

impl Sub for Date {
	type Output = Duration;

	fn sub(self, rhs: Self) -> Self::Output { (self.0 - rhs.0).into() }
}

impl Sub<i64> for Date {
	type Output = Result<Self, DateError>;

	fn sub(self, secs: i64) -> Self::Output {
		let secs = self.timestamp().checked_sub(secs).ok_or(DateError::OutOfRange)?;
		DateTime::from_timestamp(secs, self.timestamp_subsec_nanos())
			.map(Self)
			.ok_or(DateError::OutOfRange)
	}
}

impl Sub<f64> for Date {
	type Output = Result<Self, DateError>;

	fn sub(self, secs: f64) -> Self::Output { self + -secs }
}

impl Date {
	pub fn now() -> Self { Self(Utc::now()) }

	fn format(&self, pattern: &str) -> Result<String, DateError> {
		let mut output = String::new();
		self.0.format(pattern).write_to(&mut output).map_err(|_| DateError::InvalidFormat)?;
		Ok(output)
	}
}

fn parse_format(table: Table) -> mlua::Result<Locale> {
	let mut format = Locale::default();

	for pair in table.pairs::<LuaString, Value>() {
		let (key, value) = pair?;
		let key = key.to_str()?;
		if key == "hour24" {
			format.hour24 =
				value.as_boolean().ok_or_else(|| mlua::Error::runtime("hour24 must be a boolean"))?;
			continue;
		}
		let Value::String(value) = value else {
			return Err(mlua::Error::runtime("date format width must be a string"));
		};
		let width = match value.to_str()?.as_ref() {
			"numeric" => Width::Numeric,
			"2-digit" => Width::TwoDigit,
			value => return Err(mlua::Error::runtime(format!("invalid date format width: {value}"))),
		};

		match key.as_ref() {
			"year" => format.year = Some(width),
			"month" => format.month = Some(width),
			"day" => format.day = Some(width),
			"hour" => format.hour = Some(width),
			"minute" => format.minute = Some(width),
			"second" => format.second = Some(width),
			key => return Err(mlua::Error::runtime(format!("invalid date format field: {key}"))),
		}
	}

	if [format.year, format.month, format.day, format.hour, format.minute, format.second]
		.iter()
		.all(Option::is_none)
	{
		return Err(mlua::Error::runtime("date format needs at least one field"));
	}
	Ok(format)
}

impl FromLua for Date {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			Value::Integer(secs) => Ok(Self::try_from(secs)?),
			Value::Number(secs) => Ok(Self::try_from(secs)?),
			Value::String(value) => value.to_str()?.parse().into_lua_err(),
			value => Err(
				format!("expected a Date, Unix timestamp, or RFC3339 string, got {}", value.type_name())
					.into_lua_err(),
			),
		}
	}
}

impl UserData for Date {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("unix", |_, me| Ok(f64::from(*me)));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("format", |lua, me, value: Value| {
			let output = match value {
				Value::String(pattern) => me.format(&pattern.to_str()?)?.into_bytes(),
				Value::Table(options) => {
					parse_format(options)?.format(**me).map_err(|_| DateError::InvalidFormat)?
				}
				_ => return Err("expected a format string or options table".into_lua_err()),
			};
			lua.create_string(output)
		});

		methods.add_meta_method(MetaMethod::Add, |_, me, secs: f64| Ok((*me + secs)?));
		methods.add_meta_method(MetaMethod::Sub, |lua, me, value: Value| match value {
			Value::UserData(ud) => (*me - *ud.borrow::<Self>()?).into_lua(lua),
			Value::Integer(secs) => (*me - secs)?.into_lua(lua),
			Value::Number(secs) => (*me - secs)?.into_lua(lua),
			_ => Err("expected a Date or number of seconds".into_lua_err()),
		});

		methods.add_meta_method(MetaMethod::Eq, |_, me, other: UserDataRef<Self>| Ok(*me == *other));
		methods.add_meta_method(MetaMethod::Lt, |_, me, other: UserDataRef<Self>| Ok(*me < *other));
		methods.add_meta_method(MetaMethod::Le, |_, me, other: UserDataRef<Self>| Ok(*me <= *other));
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.to_rfc3339()));

		if !LOG_LEVEL.get().is_none() {
			methods.add_meta_function(MetaMethod::ToDebugString, |_, ud: AnyUserData| {
				Ok(format!("Date({:?}): {}", ud.to_pointer(), ud.borrow::<Self>()?.to_rfc3339()))
			});
		}
	}
}
