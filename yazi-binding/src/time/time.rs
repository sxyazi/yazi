use std::{ops::{Add, Deref, Sub}, str::FromStr, time::{self, SystemTime}};

use chrono::{DateTime, Datelike, Local, TimeDelta, Timelike, Utc};
use mlua::{AnyUserData, ExternalError, ExternalResult, FromLua, IntoLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef, Value};
use serde::{Deserialize, Serialize};
use yazi_ffi::locale::Locale;
use yazi_shim::log::LOG_LEVEL;

use super::{Duration, TimeError};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Time(DateTime<Utc>);

impl Deref for Time {
	type Target = DateTime<Utc>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<u32> for Time {
	fn from(secs: u32) -> Self {
		Self(DateTime::from_timestamp_nanos(i64::from(secs) * 1_000_000_000))
	}
}

impl From<SystemTime> for Time {
	fn from(time: SystemTime) -> Self { Self(time.into()) }
}

impl From<Time> for f64 {
	fn from(time: Time) -> Self {
		time.timestamp() as Self + time.timestamp_subsec_nanos() as Self / 1_000_000_000.0
	}
}

impl From<Time> for SystemTime {
	fn from(time: Time) -> Self { time.0.into() }
}

impl TryFrom<i64> for Time {
	type Error = TimeError;

	fn try_from(secs: i64) -> Result<Self, Self::Error> {
		DateTime::from_timestamp(secs, 0).map(Self).ok_or(TimeError::OutOfRange)
	}
}

impl TryFrom<f64> for Time {
	type Error = TimeError;

	fn try_from(secs: f64) -> Result<Self, Self::Error> { Self(DateTime::UNIX_EPOCH) + secs }
}

impl TryFrom<time::Duration> for Time {
	type Error = TimeError;

	fn try_from(dur: time::Duration) -> Result<Self, Self::Error> {
		DateTime::UNIX_EPOCH
			.checked_add_signed(TimeDelta::from_std(dur)?)
			.map(Self)
			.ok_or(TimeError::OutOfRange)
	}
}

impl TryFrom<Time> for time::Duration {
	type Error = TimeError;

	fn try_from(time: Time) -> Result<Self, Self::Error> {
		Ok(time.0.signed_duration_since(DateTime::UNIX_EPOCH).to_std()?)
	}
}

impl FromStr for Time {
	type Err = chrono::ParseError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		DateTime::parse_from_rfc3339(value).map(|value| Self(value.to_utc()))
	}
}

impl Add<f64> for Time {
	type Output = Result<Self, TimeError>;

	fn add(self, secs: f64) -> Self::Output {
		if !secs.is_finite() {
			return Err(TimeError::NonFinite);
		}

		let dur = TimeDelta::from_std(time::Duration::try_from_secs_f64(secs.abs())?)?;
		let dur = if secs < 0.0 { -dur } else { dur };
		self.0.checked_add_signed(dur).map(Self).ok_or(TimeError::OutOfRange)
	}
}

impl Sub for Time {
	type Output = Duration;

	fn sub(self, rhs: Self) -> Self::Output { (self.0 - rhs.0).into() }
}

impl Sub<i64> for Time {
	type Output = Result<Self, TimeError>;

	fn sub(self, secs: i64) -> Self::Output {
		TimeDelta::try_seconds(secs)
			.and_then(|dur| self.0.checked_sub_signed(dur))
			.map(Self)
			.ok_or(TimeError::OutOfRange)
	}
}

impl Sub<f64> for Time {
	type Output = Result<Self, TimeError>;

	fn sub(self, secs: f64) -> Self::Output { self + -secs }
}

impl Time {
	pub fn now() -> Self { Self(Utc::now()) }

	fn format(&self, pattern: &str) -> Result<String, TimeError> {
		let mut output = String::new();
		self.0.format(pattern).write_to(&mut output)?;
		Ok(output)
	}
}

impl FromLua for Time {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			Value::Integer(secs) => Ok(secs.try_into()?),
			Value::Number(secs) => Ok(secs.try_into()?),
			Value::String(value) => value.to_str()?.parse().into_lua_err(),
			value => Err(
				format!("expected a Time, Unix timestamp, or RFC3339 string, got {}", value.type_name())
					.into_lua_err(),
			),
		}
	}
}

impl UserData for Time {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("unix", |_, me| Ok(f64::from(*me)));
		fields.add_field_method_get("year", |_, me| Ok(me.with_timezone(&Local).year()));
		fields.add_field_method_get("month", |_, me| Ok(me.with_timezone(&Local).month()));
		fields.add_field_method_get("day", |_, me| Ok(me.with_timezone(&Local).day()));
		fields.add_field_method_get("hour", |_, me| Ok(me.with_timezone(&Local).hour()));
		fields.add_field_method_get("minute", |_, me| Ok(me.with_timezone(&Local).minute()));
		fields.add_field_method_get("second", |_, me| Ok(me.with_timezone(&Local).second()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("format", |lua, me, value: Value| {
			let output = match value {
				Value::String(s) => me.format(&s.to_str()?)?,
				v @ Value::Table(_) => Locale::from_lua(v, lua)?.format(me.0)?,
				_ => return Err("expected a format string or options table".into_lua_err()),
			};
			lua.create_string(output)
		});

		methods.add_meta_method(MetaMethod::Add, |_, me, secs: f64| Ok((*me + secs)?));
		methods.add_meta_method(MetaMethod::Sub, |lua, me, value: Value| match value {
			Value::UserData(ud) => (*me - *ud.borrow::<Self>()?).into_lua(lua),
			Value::Integer(secs) => (*me - secs)?.into_lua(lua),
			Value::Number(secs) => (*me - secs)?.into_lua(lua),
			_ => Err("expected a Time or number of seconds".into_lua_err()),
		});

		methods.add_meta_method(MetaMethod::Eq, |_, me, other: UserDataRef<Self>| Ok(*me == *other));
		methods.add_meta_method(MetaMethod::Lt, |_, me, other: UserDataRef<Self>| Ok(*me < *other));
		methods.add_meta_method(MetaMethod::Le, |_, me, other: UserDataRef<Self>| Ok(*me <= *other));
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.to_rfc3339()));

		if !LOG_LEVEL.get().is_none() {
			methods.add_meta_function(MetaMethod::ToDebugString, |_, ud: AnyUserData| {
				Ok(format!("Time({:?}): {}", ud.to_pointer(), ud.borrow::<Self>()?.to_rfc3339()))
			});
		}
	}
}
