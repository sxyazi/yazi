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
			use ratatui::style::Modifier;

			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			match value {
				mlua::Value::Boolean(true) if me.add_modifier.contains(Modifier::REVERSED) && !me.sub_modifier.contains(Modifier::REVERSED) => {
					me.bg.map($crate::Color).into_lua(lua)
				}
				mlua::Value::Nil | mlua::Value::Boolean(_) => {
					me.fg.map($crate::Color).into_lua(lua)
				}
				_ => {
					me.fg = Some($crate::Color::try_from(value)?.0);
					ud.into_lua(lua)
				}
			}
		});
		$methods.add_function_mut("bg", |lua, (ud, value): (mlua::AnyUserData, mlua::Value)| {
			use ratatui::style::Modifier;

			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			match value {
				mlua::Value::Boolean(true) if me.add_modifier.contains(Modifier::REVERSED) && !me.sub_modifier.contains(Modifier::REVERSED) => {
					me.fg.map($crate::Color).into_lua(lua)
				}
				mlua::Value::Nil | mlua::Value::Boolean(_) => {
					me.bg.map($crate::Color).into_lua(lua)
				}
				_ => {
					me.bg = Some($crate::Color::try_from(value)?.0);
					ud.into_lua(lua)
				}
			}
		});
		$methods.add_function_mut("bold", |_, (ud, remove): (mlua::AnyUserData, bool)| {
			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			if remove {
				*me = me.remove_modifier(ratatui::style::Modifier::BOLD);
			} else {
				*me = me.add_modifier(ratatui::style::Modifier::BOLD);
			}
			Ok(ud)
		});
		$methods.add_function_mut("dim", |_, (ud, remove): (mlua::AnyUserData, bool)| {
			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			if remove {
				*me = me.remove_modifier(ratatui::style::Modifier::DIM);
			} else {
				*me = me.add_modifier(ratatui::style::Modifier::DIM);
			}
			Ok(ud)
		});
		$methods.add_function_mut("italic", |_, (ud, remove): (mlua::AnyUserData, bool)| {
			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			if remove {
				*me = me.remove_modifier(ratatui::style::Modifier::ITALIC);
			} else {
				*me = me.add_modifier(ratatui::style::Modifier::ITALIC);
			}
			Ok(ud)
		});
		$methods.add_function_mut("underline", |_, (ud, remove): (mlua::AnyUserData, bool)| {
			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			if remove {
				*me = me.remove_modifier(ratatui::style::Modifier::UNDERLINED);
			} else {
				*me = me.add_modifier(ratatui::style::Modifier::UNDERLINED);
			}
			Ok(ud)
		});
		$methods.add_function_mut("blink", |_, (ud, remove): (mlua::AnyUserData, bool)| {
			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			if remove {
				*me = me.remove_modifier(ratatui::style::Modifier::SLOW_BLINK);
			} else {
				*me = me.add_modifier(ratatui::style::Modifier::SLOW_BLINK);
			}
			Ok(ud)
		});
		$methods.add_function_mut("blink_rapid", |_, (ud, remove): (mlua::AnyUserData, bool)| {
			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			if remove {
				*me = me.remove_modifier(ratatui::style::Modifier::RAPID_BLINK);
			} else {
				*me = me.add_modifier(ratatui::style::Modifier::RAPID_BLINK);
			}
			Ok(ud)
		});
		$methods.add_function_mut("reverse", |_, (ud, remove): (mlua::AnyUserData, bool)| {
			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			if remove {
				*me = me.remove_modifier(ratatui::style::Modifier::REVERSED);
			} else {
				*me = me.add_modifier(ratatui::style::Modifier::REVERSED);
			}
			Ok(ud)
		});
		$methods.add_function_mut("hidden", |_, (ud, remove): (mlua::AnyUserData, bool)| {
			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			if remove {
				*me = me.remove_modifier(ratatui::style::Modifier::HIDDEN);
			} else {
				*me = me.add_modifier(ratatui::style::Modifier::HIDDEN);
			}
			Ok(ud)
		});
		$methods.add_function_mut("crossed", |_, (ud, remove): (mlua::AnyUserData, bool)| {
			let me = &mut ud.borrow_mut::<Self>()?.$($field).+;
			if remove {
				*me = me.remove_modifier(ratatui::style::Modifier::CROSSED_OUT);
			} else {
				*me = me.add_modifier(ratatui::style::Modifier::CROSSED_OUT);
			}
			Ok(ud)
		});
		$methods.add_function_mut("reset", |_, ud: mlua::AnyUserData| {
			ud.borrow_mut::<Self>()?.$($field).+ = ratatui::style::Style::reset();
			Ok(ud)
		});
	};
}

#[macro_export]
macro_rules! impl_file_fields {
	($fields:ident) => {
		$crate::cached_field!($fields, cha, |_, me| Ok($crate::Cha(me.cha)));
		$crate::cached_field!($fields, url, |_, me| Ok($crate::Url::new(me.url_owned())));
		$crate::cached_field!($fields, link_to, |_, me| Ok(me.link_to.as_ref().map($crate::Path::new)));

		$crate::cached_field!($fields, name, |lua, me| {
			me.name().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		$crate::cached_field!($fields, path, |_, me| {
			use yazi_fs::FsUrl;
			use yazi_shared::url::AsUrl;
			Ok($crate::Path::new(me.url.as_url().unified_path()))
		});
		$crate::cached_field!($fields, cache, |_, me| {
			use yazi_fs::FsUrl;
			Ok(me.url.cache().map($crate::Path::new))
		});
	};
}

#[macro_export]
macro_rules! impl_file_methods {
	($methods:ident) => {
		$methods.add_method("hash", |_, me, ()| {
			use yazi_fs::FsHash64;
			Ok(me.hash_u64())
		});

		$methods.add_method("icon", |_, me, ()| {
			use $crate::Icon;
			// TODO: use a cache
			Ok(yazi_config::THEME.icon.matches(me).map(Icon::from))
		});
	};
}
