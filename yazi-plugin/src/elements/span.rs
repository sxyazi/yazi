use mlua::{AnyUserData, ExternalError, FromLua, Lua, Table, UserData, UserDataMethods, Value};

use super::Style;

#[derive(Clone, FromLua)]
pub struct Span(pub(super) ratatui::text::Span<'static>);

impl Span {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.raw_set(
			"Span",
			lua.create_function(|_, content: mlua::String| {
				Ok(Self(ratatui::text::Span::raw(content.to_string_lossy().into_owned())))
			})?,
		)
	}
}

impl UserData for Span {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		crate::impl_style_shorthands!(methods, 0.style);

		methods.add_function("style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.0.style = match value {
				Value::Nil => ratatui::style::Style::default(),
				Value::Table(tb) => Style::try_from(tb)?.0,
				Value::UserData(ud) => ud.borrow::<Style>()?.0,
				_ => return Err("expected a Style or Table or nil".into_lua_err()),
			};
			Ok(ud)
		});
	}
}
