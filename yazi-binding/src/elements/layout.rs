use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};

use super::{Constraint, Rect};

const HORIZONTAL: bool = true;
const VERTICAL: bool = false;

#[derive(Clone, Default)]
pub struct Layout {
	direction:   bool,
	margin:      Option<ratatui::layout::Margin>,
	constraints: Vec<ratatui::layout::Constraint>,
}

impl Layout {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, _: Table| Ok(Self::default()))?;

		let layout = lua.create_table_from([("HORIZONTAL", HORIZONTAL), ("VERTICAL", VERTICAL)])?;
		layout.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		layout.into_lua(lua)
	}
}

impl UserData for Layout {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_function_mut("direction", |_, (ud, value): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.direction = value;
			Ok(ud)
		});
		methods.add_function_mut("margin", |_, (ud, value): (AnyUserData, u16)| {
			ud.borrow_mut::<Self>()?.margin = Some(ratatui::layout::Margin::new(value, value));
			Ok(ud)
		});
		methods.add_function_mut("margin_h", |_, (ud, value): (AnyUserData, u16)| {
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
		methods.add_function_mut("margin_v", |_, (ud, value): (AnyUserData, u16)| {
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
		methods.add_function_mut("constraints", |_, (ud, value): (AnyUserData, Vec<Constraint>)| {
			ud.borrow_mut::<Self>()?.constraints = value.into_iter().map(Into::into).collect();
			Ok(ud)
		});
		methods.add_method("split", |lua, me, value: Rect| {
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

			lua.create_sequence_from(layout.split(*value).iter().map(|chunk| Rect::from(*chunk)))
		});
	}
}
