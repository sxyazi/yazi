#[macro_export]
macro_rules! runtime {
	($lua:ident) => {{
		use mlua::ExternalError;
		$lua.app_data_ref::<$crate::Runtime>().ok_or_else(|| "Runtime not found".into_lua_err())
	}};
}

#[macro_export]
macro_rules! runtime_mut {
	($lua:ident) => {{
		use mlua::ExternalError;
		$lua.app_data_mut::<$crate::Runtime>().ok_or_else(|| "Runtime not found".into_lua_err())
	}};
}

#[macro_export]
macro_rules! deprecate {
	($lua:ident, $tt:tt) => {{
		static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
		if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
			let id = match $crate::runtime!($lua)?.current() {
				Some(id) => &format!("`{id}.yazi` plugin"),
				None => "`init.lua` config",
			};
			yazi_macro::emit!(Call(
				yazi_macro::relay!(app:deprecate).with("content", format!($tt, id))
			));
		}
	}};
}

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
macro_rules! impl_area_method {
	($methods:ident) => {
		$methods.add_function_mut(
			"area",
			|lua, (ud, area): (mlua::AnyUserData, Option<mlua::AnyUserData>)| {
				use mlua::IntoLua;
				if let Some(v) = area {
					ud.borrow_mut::<Self>()?.area = $crate::elements::Area::try_from(v)?;
					ud.into_lua(lua)
				} else {
					ud.borrow::<Self>()?.area.into_lua(lua)
				}
			},
		);
	};
}

#[macro_export]
macro_rules! impl_style_method {
	($methods:ident, $($field:tt).+) => {
		$methods.add_function_mut("style", |_, (ud, value): (mlua::AnyUserData, mlua::Value)| {
			ud.borrow_mut::<Self>()?.$($field).+ = $crate::Style::try_from(value)?.0;
			Ok(ud)
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

#[macro_export]
macro_rules! impl_file_fields {
	($fields:ident) => {
		$crate::cached_field!($fields, cha, |_, me| Ok($crate::Cha(me.cha)));
		$crate::cached_field!($fields, url, |_, me| Ok($crate::Url::new(me.url_owned())));
		$crate::cached_field!($fields, link_to, |_, me| Ok(me.link_to.clone().map($crate::Url::new)));

		$crate::cached_field!($fields, name, |lua, me| {
			Some(me.name())
				.filter(|s| !s.is_empty())
				.map(|s| lua.create_string(s.as_encoded_bytes()))
				.transpose()
		});
	};
}

#[macro_export]
macro_rules! impl_file_methods {
	($methods:ident) => {
		$methods.add_method("hash", |_, me, ()| Ok(me.hash_u64()));

		$methods.add_method("icon", |_, me, ()| {
			use $crate::Icon;
			// TODO: use a cache
			Ok(yazi_config::THEME.icon.matches(me).map(Icon::from))
		});
	};
}
