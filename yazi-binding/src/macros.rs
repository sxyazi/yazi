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

#[macro_export]
macro_rules! impl_style_shorthands {
	($methods:ident, $($field:tt).+) => {
		$methods.add_function_mut("fg", |lua, (ud, value): (mlua::AnyUserData, mlua::Value)| {
			match value {
				mlua::Value::Nil => {
					ud.borrow::<Self>()?.$($field).+.fg.map($crate::Color).into_lua(lua)
				},
				_ => {
					ud.borrow_mut::<Self>()?.$($field).+.fg = Some($crate::Color::try_from(value)?.0);
					ud.into_lua(lua)
				}
			}
		});
		$methods.add_function_mut("bg", |lua, (ud, value): (mlua::AnyUserData, mlua::Value)| {
			match value {
				mlua::Value::Nil => {
					ud.borrow::<Self>()?.$($field).+.bg.map($crate::Color).into_lua(lua)
				}
				_ => {
					ud.borrow_mut::<Self>()?.$($field).+.bg = Some($crate::Color::try_from(value)?.0);
					ud.into_lua(lua)
				}
			}
		});
		$methods.add_function_mut("bold", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+.add_modifier |= ratatui::style::Modifier::BOLD;
			Ok(ud)
		});
		$methods.add_function_mut("dim", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+.add_modifier |= ratatui::style::Modifier::DIM;
			Ok(ud)
		});
		$methods.add_function_mut("italic", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+.add_modifier |= ratatui::style::Modifier::ITALIC;
			Ok(ud)
		});
		$methods.add_function_mut("underline", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+.add_modifier |= ratatui::style::Modifier::UNDERLINED;
			Ok(ud)
		});
		$methods.add_function_mut("blink", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+.add_modifier |= ratatui::style::Modifier::SLOW_BLINK;
			Ok(ud)
		});
		$methods.add_function_mut("blink_rapid", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+.add_modifier |= ratatui::style::Modifier::RAPID_BLINK;
			Ok(ud)
		});
		$methods.add_function_mut("reverse", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+.add_modifier |= ratatui::style::Modifier::REVERSED;
			Ok(ud)
		});
		$methods.add_function_mut("hidden", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+.add_modifier |= ratatui::style::Modifier::HIDDEN;
			Ok(ud)
		});
		$methods.add_function_mut("crossed", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+.add_modifier |= ratatui::style::Modifier::CROSSED_OUT;
			Ok(ud)
		});
		$methods.add_function_mut("reset", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+.add_modifier = ratatui::style::Modifier::empty();
			Ok(ud)
		});
	};
}
