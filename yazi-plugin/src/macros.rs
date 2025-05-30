#[macro_export]
macro_rules! impl_style_method {
	($methods:ident, $($field:tt).+) => {
		$methods.add_function_mut("style", |_, (ud, value): (mlua::AnyUserData, mlua::Value)| {
			ud.borrow_mut::<Self>()?.$($field).+ = $crate::elements::Style::try_from(value)?.0;
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
macro_rules! impl_style_shorthands {
	($methods:ident, $($field:tt).+) => {
		$methods.add_function_mut("fg", |_, (ud, color): (mlua::AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.$($field).+.fg = yazi_shared::theme::Color::try_from(color).ok().map(Into::into);
			Ok(ud)
		});
		$methods.add_function_mut("bg", |_, (ud, color): (mlua::AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.$($field).+.bg = yazi_shared::theme::Color::try_from(color).ok().map(Into::into);
			Ok(ud)
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
		$methods.add_method("hash", |_, me, ()| Ok(me.hash()));

		$methods.add_method("icon", |_, me, ()| {
			use yazi_shared::theme::IconCache;
			use $crate::bindings::Icon;

			Ok(match me.icon.get() {
				IconCache::Missing => {
					let matched = yazi_config::THEME.icon.matches(me);
					me.icon.set(matched.map_or(IconCache::Undefined, IconCache::Icon));
					matched.map(Icon::from)
				}
				IconCache::Undefined => None,
				IconCache::Icon(cached) => Some(Icon::from(cached)),
			})
		});
	};
}

#[macro_export]
macro_rules! deprecate {
	($lua:ident, $tt:tt) => {{
		let id = match $lua.named_registry_value::<$crate::RtRef>("ir")?.current() {
			Some(id) => &format!("`{id}.yazi` plugin"),
			None => "`init.lua` config",
		};
		yazi_proxy::deprecate!(format!($tt, id));
	}};
}
