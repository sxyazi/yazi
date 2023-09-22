use mlua::{AnyUserData, FromLua, Lua, Table, UserData, UserDataMethods, Value};
use ratatui::layout;

use crate::LUA;

// --- Rect
#[derive(Clone, Copy)]
pub struct Rect(layout::Rect);

impl From<layout::Rect> for Rect {
	fn from(value: layout::Rect) -> Self { Self(value) }
}

impl<'lua> FromLua<'lua> for Rect {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Rect",
				message: Some("expected a Rect".to_string()),
			}),
		}
	}
}

impl UserData for Rect {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("x", |_, me| Ok(me.0.x));
		fields.add_field_method_get("y", |_, me| Ok(me.0.y));
		fields.add_field_method_get("width", |_, me| Ok(me.0.width));
		fields.add_field_method_get("height", |_, me| Ok(me.0.height));
	}
}

// --- Constraint
#[derive(Clone, Copy)]
pub struct Constraint(layout::Constraint);

impl Constraint {
	pub(super) fn install() -> mlua::Result<()> {
		let globals = LUA.globals();
		let yazi = globals.get::<_, Table>("yazi")?;

		let constraint = LUA.create_table()?;
		constraint.set(
			"Percentage",
			LUA.create_function(|_, n: u16| Ok(Constraint(layout::Constraint::Percentage(n))))?,
		)?;
		constraint.set(
			"Ratio",
			LUA
				.create_function(|_, (a, b): (u32, u32)| Ok(Constraint(layout::Constraint::Ratio(a, b))))?,
		)?;
		constraint.set(
			"Length",
			LUA.create_function(|_, n: u16| Ok(Constraint(layout::Constraint::Length(n))))?,
		)?;
		constraint
			.set("Max", LUA.create_function(|_, n: u16| Ok(Constraint(layout::Constraint::Max(n))))?)?;
		constraint
			.set("Min", LUA.create_function(|_, n: u16| Ok(Constraint(layout::Constraint::Min(n))))?)?;

		yazi.set("Constraint", constraint)
	}
}

impl<'lua> FromLua<'lua> for Constraint {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Constraint",
				message: Some("expected a Constraint".to_string()),
			}),
		}
	}
}

impl UserData for Constraint {}

// --- Layout
#[derive(Clone, Default)]
pub struct Layout {
	direction:   bool,
	margin:      Option<layout::Margin>,
	constraints: Vec<layout::Constraint>,
}

impl Layout {
	pub(super) fn install() -> mlua::Result<()> {
		let globals = LUA.globals();

		let yazi = globals.get::<_, Table>("yazi")?;
		yazi.set("Layout", LUA.create_function(|_, ()| Ok(Self::default()))?)
	}
}

impl<'lua> FromLua<'lua> for Layout {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Layout",
				message: Some("expected a Layout".to_string()),
			}),
		}
	}
}

impl UserData for Layout {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("direction", |_, (ud, value): (AnyUserData, bool)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				me.direction = value;
			}
			Ok(ud)
		});
		methods.add_function("margin", |_, (ud, value): (AnyUserData, u16)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				me.margin = Some(layout::Margin::new(value, value));
			}
			Ok(ud)
		});
		methods.add_function("margin_h", |_, (ud, value): (AnyUserData, u16)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				if let Some(margin) = &mut me.margin {
					margin.horizontal = value;
				} else {
					me.margin = Some(layout::Margin::new(value, 0));
				}
			}
			Ok(ud)
		});
		methods.add_function("margin_v", |_, (ud, value): (AnyUserData, u16)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				if let Some(margin) = &mut me.margin {
					margin.vertical = value;
				} else {
					me.margin = Some(layout::Margin::new(0, value));
				}
			}
			Ok(ud)
		});
		methods.add_function("constraints", |_, (ud, value): (AnyUserData, Vec<Constraint>)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				me.constraints = value.into_iter().map(|c| c.0).collect();
			}
			Ok(ud)
		});
		methods.add_function("split", |_, (ud, value): (AnyUserData, Rect)| {
			let me = ud.borrow::<Self>()?;

			let mut layout = layout::Layout::new()
				.direction(if me.direction {
					layout::Direction::Horizontal
				} else {
					layout::Direction::Vertical
				})
				.constraints(me.constraints.as_slice());

			if let Some(margin) = me.margin {
				layout = layout.horizontal_margin(margin.horizontal);
				layout = layout.vertical_margin(margin.vertical);
			}

			let chunks: Vec<Rect> = layout.split(value.0).iter().copied().map(Rect).collect();
			Ok(chunks)
		});
	}
}
