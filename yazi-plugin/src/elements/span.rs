use mlua::{FromLua, Lua, Table, UserData, UserDataMethods};
use unicode_width::UnicodeWidthChar;

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
		crate::impl_style_method!(methods, 0.style);
		crate::impl_style_shorthands!(methods, 0.style);

		methods.add_method("visible", |_, me, ()| {
			Ok(me.0.content.chars().any(|c| c.width().unwrap_or(0) > 0))
		});
	}
}
