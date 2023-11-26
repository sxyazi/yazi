use mlua::{AnyUserData, Lua, Table, UserData, UserDataMethods};

use super::{Constraint, Rect, RectRef};
use crate::bindings::Cast;

#[derive(Clone, Default)]
pub struct Layout {
	direction:   bool,
	margin:      Option<ratatui::layout::Margin>,
	constraints: Vec<ratatui::layout::Constraint>,
}

impl Layout {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.set("Layout", lua.create_function(|_, ()| Ok(Self::default()))?)
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
		methods.add_function("split", |lua, (ud, value): (AnyUserData, RectRef)| {
			let me = ud.borrow::<Self>()?;

			let mut layout = ratatui::layout::Layout::new()
				.direction(if me.direction {
					ratatui::layout::Direction::Vertical
				} else {
					ratatui::layout::Direction::Horizontal
				})
				.constraints(me.constraints.as_slice());

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
