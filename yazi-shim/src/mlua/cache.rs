use mlua::{Error::UserDataDestructed, Value};

pub(super) fn is_alive(v: &Value) -> bool {
	if let Value::UserData(ud) = v {
		!matches!(ud.borrow::<()>(), Err(UserDataDestructed))
	} else {
		true
	}
}
