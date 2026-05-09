use std::ops::Deref;

use mlua::{ExternalError, FromLua, IntoLua, Lua, LuaSerdeExt, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui::style::Modifier;
use yazi_shim::cell::SyncCell;

use crate::{SER_OPT, elements::Color};

#[derive(Clone, Copy, Default)]
pub struct Style(pub ratatui::style::Style);

impl Deref for Style {
	type Target = ratatui::style::Style;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<yazi_config::Style> for Style {
	fn from(value: yazi_config::Style) -> Self { Self(value.into()) }
}

impl From<Style> for ratatui::style::Style {
	fn from(value: Style) -> Self { value.0 }
}

impl From<&SyncCell<yazi_config::Style>> for Style {
	fn from(value: &SyncCell<yazi_config::Style>) -> Self { value.get().into() }
}

impl Style {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, style): (Table, Self)| Ok(style))?;

		let style = lua.create_table()?;
		style.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		style.into_lua(lua)
	}
}

impl FromLua for Style {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(Self(match value {
			Value::Nil => Default::default(),
			Value::UserData(ud) => ud.borrow::<Self>()?.0,
			_ => Err("expected a Style or nil".into_lua_err())?,
		}))
	}
}

impl UserData for Style {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("fg", |lua, me, value: Value| match value {
			Value::Boolean(true) if me.has_modifier(Modifier::REVERSED) => me.bg.map(Color).into_lua(lua),
			Value::Nil | Value::Boolean(_) => me.fg.map(Color).into_lua(lua),
			_ => Self(me.fg(Color::from_lua(value, lua)?.0)).into_lua(lua),
		});

		methods.add_method("bg", |lua, me, value: Value| match value {
			Value::Boolean(true) if me.has_modifier(Modifier::REVERSED) => me.fg.map(Color).into_lua(lua),
			Value::Nil | Value::Boolean(_) => me.bg.map(Color).into_lua(lua),
			_ => Self(me.bg(Color::from_lua(value, lua)?.0)).into_lua(lua),
		});

		methods.add_method("bold", |_, me, remove: bool| {
			Ok(Self(if remove {
				me.remove_modifier(Modifier::BOLD)
			} else {
				me.add_modifier(Modifier::BOLD)
			}))
		});

		methods.add_method("dim", |_, me, remove: bool| {
			Ok(Self(if remove {
				me.remove_modifier(Modifier::DIM)
			} else {
				me.add_modifier(Modifier::DIM)
			}))
		});

		methods.add_method("italic", |_, me, remove: bool| {
			Ok(Self(if remove {
				me.remove_modifier(Modifier::ITALIC)
			} else {
				me.add_modifier(Modifier::ITALIC)
			}))
		});

		methods.add_method("underline", |_, me, remove: bool| {
			Ok(Self(if remove {
				me.remove_modifier(Modifier::UNDERLINED)
			} else {
				me.add_modifier(Modifier::UNDERLINED)
			}))
		});

		methods.add_method("blink", |_, me, remove: bool| {
			Ok(Self(if remove {
				me.remove_modifier(Modifier::SLOW_BLINK)
			} else {
				me.add_modifier(Modifier::SLOW_BLINK)
			}))
		});

		methods.add_method("blink_rapid", |_, me, remove: bool| {
			Ok(Self(if remove {
				me.remove_modifier(Modifier::RAPID_BLINK)
			} else {
				me.add_modifier(Modifier::RAPID_BLINK)
			}))
		});

		methods.add_method("reverse", |_, me, remove: bool| {
			Ok(Self(if remove {
				me.remove_modifier(Modifier::REVERSED)
			} else {
				me.add_modifier(Modifier::REVERSED)
			}))
		});

		methods.add_method("hidden", |_, me, remove: bool| {
			Ok(Self(if remove {
				me.remove_modifier(Modifier::HIDDEN)
			} else {
				me.add_modifier(Modifier::HIDDEN)
			}))
		});

		methods.add_method("crossed", |_, me, remove: bool| {
			Ok(Self(if remove {
				me.remove_modifier(Modifier::CROSSED_OUT)
			} else {
				me.add_modifier(Modifier::CROSSED_OUT)
			}))
		});

		methods.add_method("patch", |_, me, other: Self| Ok(Self(me.patch(other))));

		methods
			.add_method("raw", |lua, me, ()| lua.to_value_with(&yazi_config::Style::from(me.0), SER_OPT));
	}
}
