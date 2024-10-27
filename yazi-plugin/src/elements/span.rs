use mlua::{ExternalError, FromLua, Lua, Table, UserData, UserDataMethods, Value};
use unicode_width::UnicodeWidthChar;

const EXPECTED: &str = "expected a string or ui.Span";

#[derive(Clone, FromLua)]
pub struct Span(pub(super) ratatui::text::Span<'static>);

impl Span {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.raw_set("Span", lua.create_function(|_, value: Value| Span::try_from(value))?)
	}
}

impl TryFrom<Value> for Span {
	type Error = mlua::Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		Ok(Self(match value {
			Value::String(s) => s.to_string_lossy().into(),
			Value::UserData(ud) => {
				if let Ok(Span(span)) = ud.take() {
					span
				} else {
					Err(EXPECTED.into_lua_err())?
				}
			}
			_ => Err(EXPECTED.into_lua_err())?,
		}))
	}
}

impl UserData for Span {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_style_method!(methods, 0.style);
		crate::impl_style_shorthands!(methods, 0.style);

		methods.add_method("visible", |_, Span(me), ()| {
			Ok(me.content.chars().any(|c| c.width().unwrap_or(0) > 0))
		});
	}
}
