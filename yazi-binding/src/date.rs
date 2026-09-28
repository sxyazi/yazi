use chrono::{DateTime, Datelike, Timelike, Utc};
use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, Lua, LuaString, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef, Value};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Date(DateTime<Utc>);

#[derive(Debug, Error)]
pub enum DateError {
	#[error("timestamp must be finite")]
	NonFinite,
	#[error("timestamp is out of range")]
	OutOfRange,
	#[error("invalid date format")]
	InvalidFormat,
}

impl Date {
	pub fn now() -> Self { Self(Utc::now()) }

	pub fn parse(value: &str) -> Result<Self, chrono::ParseError> {
		DateTime::parse_from_rfc3339(value).map(|value| Self(value.to_utc()))
	}

	pub fn from_unix(seconds: f64) -> Result<Self, DateError> {
		if !seconds.is_finite() {
			return Err(DateError::NonFinite);
		}

		let whole = seconds.floor();
		let nanos = ((seconds - whole) * 1_000_000_000.0).round();
		let (whole, nanos) = if nanos >= 1_000_000_000.0 { (whole + 1.0, 0.0) } else { (whole, nanos) };

		DateTime::from_timestamp(whole as i64, nanos as u32).map(Self).ok_or(DateError::OutOfRange)
	}

	pub fn unix(self) -> f64 {
		self.0.timestamp() as f64 + self.0.timestamp_subsec_nanos() as f64 / 1_000_000_000.0
	}

	fn shift(self, seconds: f64) -> Result<Self, DateError> { Self::from_unix(self.unix() + seconds) }

	fn format(&self, pattern: Option<&str>) -> Result<String, DateError> {
		let Some(pattern) = pattern else { return Ok(self.0.to_rfc3339()) };

		let mut output = String::new();
		self.0.format(pattern).write_to(&mut output).map_err(|_| DateError::InvalidFormat)?;
		Ok(output)
	}
}

impl FromLua for Date {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			Value::Integer(seconds) => Self::from_unix(seconds as f64).map_err(|e| e.into_lua_err()),
			Value::Number(seconds) => Self::from_unix(seconds).map_err(|e| e.into_lua_err()),
			Value::String(value) => Self::parse(&value.to_str()?).map_err(|e| e.into_lua_err()),
			value => Err(
				format!("expected a Date, Unix timestamp, or RFC3339 string, got {}", value.type_name())
					.into_lua_err(),
			),
		}
	}
}

impl UserData for Date {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("unix", |_, me| Ok(me.unix()));
		fields.add_field_method_get("year", |_, me| Ok(me.0.year()));
		fields.add_field_method_get("month", |_, me| Ok(me.0.month()));
		fields.add_field_method_get("day", |_, me| Ok(me.0.day()));
		fields.add_field_method_get("hour", |_, me| Ok(me.0.hour()));
		fields.add_field_method_get("minute", |_, me| Ok(me.0.minute()));
		fields.add_field_method_get("second", |_, me| Ok(me.0.second()));
		fields.add_field_method_get("nanosecond", |_, me| Ok(me.0.nanosecond()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("format", |lua, me, pattern: Option<LuaString>| {
			let pattern = pattern.as_ref().map(|s| s.to_str()).transpose()?;
			me.format(pattern.as_deref()).map_err(|e| e.into_lua_err()).and_then(|s| lua.create_string(s))
		});

		methods.add_meta_method(MetaMethod::Add, |_, me, seconds: f64| {
			me.shift(seconds).map_err(|e| e.into_lua_err())
		});
		methods.add_meta_method(MetaMethod::Sub, |lua, me, value: Value| {
			let seconds = match value {
				Value::UserData(ud) => return (me.unix() - ud.borrow::<Self>()?.unix()).into_lua(lua),
				Value::Integer(seconds) => seconds as f64,
				Value::Number(seconds) => seconds,
				_ => return Err("expected a Date or number of seconds".into_lua_err()),
			};
			me.shift(-seconds).map_err(|e| e.into_lua_err()).and_then(|date| date.into_lua(lua))
		});

		methods.add_meta_method(MetaMethod::Eq, |_, me, other: UserDataRef<Self>| Ok(*me == *other));
		methods.add_meta_method(MetaMethod::Lt, |_, me, other: UserDataRef<Self>| Ok(*me < *other));
		methods.add_meta_method(MetaMethod::Le, |_, me, other: UserDataRef<Self>| Ok(*me <= *other));
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.0.to_rfc3339()));

		if !yazi_shim::log::LOG_LEVEL.get().is_none() {
			methods.add_meta_function(MetaMethod::ToDebugString, |_, ud: AnyUserData| {
				Ok(format!("Date({:?}): {}", ud.to_pointer(), ud.borrow::<Self>()?.0.to_rfc3339()))
			});
		}
	}
}
