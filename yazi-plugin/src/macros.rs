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
		use mlua::IntoLua;
		use $crate::elements::Rect;

		$methods.add_function_mut("area", |lua, (ud, area): (mlua::AnyUserData, Option<Rect>)| {
			if let Some(r) = area {
				ud.borrow_mut::<Self>()?.area = r;
				ud.into_lua(lua)
			} else {
				ud.borrow::<Self>()?.area.into_lua(lua)
			}
		});
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
