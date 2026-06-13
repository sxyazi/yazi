use std::sync::Arc;

use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use parking_lot::Mutex;
use ratatui::widgets::Widget;

use super::Area;
use crate::{elements::Spatial, impl_area_method};

const EXPECTED: &str = "expected a table or a Input";

#[derive(Clone, Debug, Default)]
pub struct Input {
	pub focus: bool,

	inner: Arc<Mutex<yazi_widgets::input::Input>>,
}

impl Input {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, input): (Table, Self)| Ok(input))?;

		let input = lua.create_table()?;
		input.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;
		input.into_lua(lua)
	}
}

impl From<&Input> for Arc<Mutex<yazi_widgets::input::Input>> {
	fn from(input: &Input) -> Self { input.inner.clone() }
}

impl TryFrom<&AnyUserData> for Input {
	type Error = mlua::Error;

	fn try_from(value: &AnyUserData) -> Result<Self, Self::Error> {
		Ok(value.borrow::<Self>()?.clone())
	}
}

impl TryFrom<Table> for Input {
	type Error = mlua::Error;

	fn try_from(tb: Table) -> Result<Self, Self::Error> {
		let input = yazi_widgets::input::Input::new(yazi_widgets::input::InputOpt {
			cfg: yazi_config::popup::InputCfg {
				title:      String::new(),
				value:      tb.raw_get("value")?,
				cursor:     None,
				obscure:    false,
				position:   Default::default(),
				realtime:   false,
				completion: false,
			},
			tx:  None,
		})?;

		Ok(Self { inner: Arc::new(Mutex::new(input)), ..Default::default() })
	}
}

impl Spatial for Input {
	fn area(&self) -> Area { self.inner.lock().pos.into() }

	fn set_area(&mut self, area: Area) {
		self.inner.lock().repos(match area {
			Area::Rect(rect) => yazi_config::popup::Position {
				origin: yazi_config::popup::Origin::TopLeft,
				offset: yazi_config::popup::Offset {
					x:      rect.x as i16,
					y:      rect.y as i16,
					width:  rect.width,
					height: rect.height,
				},
			},
			Area::Pos(pos) => pos.into(),
		});
	}
}

impl Widget for Input {
	fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
	where
		Self: Sized,
	{
		(&self).render(rect, buf);
	}
}

impl Widget for &Input {
	fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
	where
		Self: Sized,
	{
		self.inner.lock().render(rect, buf)
	}
}

impl FromLua for Input {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(tb) => Self::try_from(tb),
			_ => Err(EXPECTED.into_lua_err()),
		}
	}
}

impl UserData for Input {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		impl_area_method!(methods);

		methods.add_function("focus", |lua, (ud, focus): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.focus = focus;
			ud.into_lua(lua)
		});
	}
}
