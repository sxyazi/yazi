#[macro_export]
macro_rules! impl_style_method {
	($methods:ident, $($field:tt).+) => {
		$methods.add_function_mut("style", |_, (ud, value): (mlua::AnyUserData, mlua::Value)| {
			ud.borrow_mut::<Self>()?.$($field).+ = yazi_binding::Style::try_from(value)?.0;
			Ok(ud)
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
macro_rules! impl_file_fields {
	($fields:ident) => {
		yazi_binding::cached_field!($fields, cha, |_, me| Ok($crate::bindings::Cha(me.cha)));
		yazi_binding::cached_field!($fields, url, |_, me| Ok(yazi_binding::Url::new(me.url_owned())));
		yazi_binding::cached_field!($fields, link_to, |_, me| Ok(
			me.link_to.clone().map(yazi_binding::Url::new)
		));

		yazi_binding::cached_field!($fields, name, |lua, me| {
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
			use yazi_binding::Icon;
			// TODO: use a cache
			Ok(yazi_config::THEME.icon.matches(me).map(Icon::from))
		});
	};
}

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
		let id = match $crate::runtime!($lua)?.current() {
			Some(id) => &format!("`{id}.yazi` plugin"),
			None => "`init.lua` config",
		};
		yazi_proxy::deprecate!(format!($tt, id));
	}};
}
