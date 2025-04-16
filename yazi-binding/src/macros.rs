#[macro_export]
macro_rules! cached_field {
	($fields:ident, $key:ident, $value:expr) => {
		$fields.add_field_function_get(stringify!($key), |lua, ud| {
			use mlua::{Error::UserDataDestructed, IntoLua, Lua, Result, Value, Value::UserData};
			ud.borrow_mut_scoped::<Self, Result<Value>>(|me| match paste::paste! { &me.[<v_ $key>] } {
				Some(v) if !v.is_userdata() => Ok(v.clone()),
				Some(v @ UserData(ud)) if !matches!(ud.borrow::<()>(), Err(UserDataDestructed)) => {
					Ok(v.clone())
				}
				_ => {
					let v = ($value as fn(&Lua, &Self) -> Result<_>)(lua, me)?.into_lua(lua)?;
					paste::paste! { me.[<v_ $key>] = Some(v.clone()) };
					Ok(v)
				}
			})?
		});
	};
}
