use chrono::TimeDelta;
use mlua::{UserData, UserDataFields};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Duration(TimeDelta);

impl From<TimeDelta> for Duration {
	fn from(duration: TimeDelta) -> Self { Self(duration) }
}

impl UserData for Duration {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("secs", |_, me| Ok(me.0.num_seconds()));
		fields.add_field_method_get("nanos", |_, me| Ok(me.0.subsec_nanos()));
	}
}
