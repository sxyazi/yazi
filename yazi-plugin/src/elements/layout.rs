use mlua::{AnyUserData, IntoLua, Lua, Table, UserData, UserDataMethods};

use super::{Constraint, Rect, RectRef};
use crate::bindings::Cast;

const HORIZONTAL: bool = true;
const VERTICAL: bool = false;

#[derive(Clone, Default)]
pub struct Layout {
	direction:   bool,
	margin:      Option<ratatui::layout::Margin>,
	constraints: Vec<ratatui::layout::Constraint>,
}

impl Layout {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, _: Table| Ok(Self::default()))?;

		let layout = lua.create_table_from([
			("HORIZONTAL", HORIZONTAL.into_lua(lua)?),
			("VERTICAL", VERTICAL.into_lua(lua)?),
		])?;

		layout.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.set("Layout", layout)
	}
}

impl UserData for Layout {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("direction", |_, (ud, value): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.direction = value;
			Ok(ud)
		});
		methods.add_function("margin", |_, (ud, value): (AnyUserData, u16)| {
			ud.borrow_mut::<Self>()?.margin = Some(ratatui::layout::Margin::new(value, value));
			Ok(ud)
		});
		methods.add_function("margin_h", |_, (ud, value): (AnyUserData, u16)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				if let Some(margin) = &mut me.margin {
					margin.horizontal = value;
				} else {
					me.margin = Some(ratatui::layout::Margin::new(value, 0));
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
					me.margin = Some(ratatui::layout::Margin::new(0, value));
				}
			}
			Ok(ud)
		});
		methods.add_function("constraints", |_, (ud, value): (AnyUserData, Vec<Constraint>)| {
			ud.borrow_mut::<Self>()?.constraints = value.into_iter().map(|c| c.0).collect();
			Ok(ud)
		});
		methods.add_method("split", |lua, me, value: RectRef| {
			let mut layout = ratatui::layout::Layout::new(
				if me.direction == VERTICAL {
					ratatui::layout::Direction::Vertical
				} else {
					ratatui::layout::Direction::Horizontal
				},
				&me.constraints,
			);

			if let Some(margin) = me.margin {
				layout = layout.horizontal_margin(margin.horizontal);
				layout = layout.vertical_margin(margin.vertical);
			}

			let mut chunks = Vec::with_capacity(me.constraints.len());
			for chunk in &*layout.split(*value) {
				chunks.push(Rect::cast(lua, *chunk)?);
			}
			Ok(chunks)
		});
	}
}
