use std::borrow::Cow;

use mlua::{AnyUserData, ExternalError, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use unicode_width::UnicodeWidthChar;

const EXPECTED: &str = "expected a string or Span";

pub struct Span(pub(super) ratatui::text::Span<'static>);

impl Span {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, value): (Table, Value)| Span::try_from(value))?;

		let span = lua.create_table()?;
		span.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		span.into_lua(lua)
	}

	pub(super) fn truncate(&mut self, max: usize) -> usize {
		if max < 1 {
			match &mut self.0.content {
				Cow::Borrowed(_) => self.0.content = Cow::Borrowed(""),
				Cow::Owned(s) => s.clear(),
			}
			return 0;
		}

		let mut adv = 0;
		let mut last;
		for (i, c) in self.0.content.char_indices() {
			(last, adv) = (adv, adv + c.width().unwrap_or(0));
			if adv < max {
				continue;
			} else if adv == max && self.0.content[i..].chars().nth(1).is_none() {
				return max;
			}
			match &mut self.0.content {
				Cow::Borrowed(s) => self.0.content = format!("{}…", &s[..i]).into(),
				Cow::Owned(s) => {
					s.truncate(i);
					s.push('…');
				}
			}
			return last + 1;
		}
		adv
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
		methods.add_function_mut("truncate", |_, (ud, t): (AnyUserData, Table)| {
			ud.borrow_mut::<Self>()?.truncate(t.raw_get("max")?);
			Ok(ud)
		});
	}
}
