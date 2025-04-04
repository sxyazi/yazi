#[macro_export]
macro_rules! cached_field {
	($fields:ident, $key:ident, $value:expr) => {
		$fields.add_field_function_get(stringify!($key), |lua, ud| {
			use mlua::IntoLua;
			ud.borrow_mut_scoped::<Self, mlua::Result<mlua::Value>>(|me| {
				match paste::paste! { &me.[<v_ $key>] } {
					Some(v) => Ok(v.clone()),
					None => {
						let v: mlua::Result<_> = $value(lua, me);
						let v = v?.into_lua(lua)?;
						paste::paste! { me.[<v_ $key>] = Some(v.clone()) };
						Ok(v)
					}
				}
			})?
		});
	};
}
