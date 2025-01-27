use mlua::{ExternalError, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use unicode_width::UnicodeWidthChar;

const EXPECTED: &str = "expected a string or Span";

pub struct Span(pub(super) ratatui::text::Span<'static>);

impl Span {
	pub fn compose(lua: &Lua) -> mlua::Result<Table> {
		let new = lua.create_function(|_, (_, value): (Table, Value)| Span::try_from(value))?;

		let span = lua.create_table()?;
		span.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		Ok(span)
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
