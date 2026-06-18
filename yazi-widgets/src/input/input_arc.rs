use std::sync::Arc;

use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use parking_lot::Mutex;
use ratatui::widgets::Widget;
use yazi_binding::{elements::{Area, Spatial}, impl_area_method, position::{Offset, Origin, Position}};

use crate::input::{Input, InputOpt, InputStyles};

#[derive(Clone, Debug, Default)]
pub struct InputArc {
	pub focus: bool,

	inner: Arc<Mutex<Input>>,
}

impl InputArc {
	pub fn compose(lua: &Lua, styles: InputStyles) -> mlua::Result<Value> {
		let new = lua.create_function(move |_, (_, mut opt): (Table, InputOpt)| {
			opt.styles.normal = opt.styles.normal.or(styles.normal);
			opt.styles.selected = opt.styles.selected.or(styles.selected);
			Ok(Self::from(Input::new(opt)?))
		})?;

		let input = lua.create_table()?;
		input.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;
		input.into_lua(lua)
	}
}

impl From<Input> for InputArc {
	fn from(input: Input) -> Self {
		Self { inner: Arc::new(Mutex::new(input)), ..Default::default() }
	}
}

impl From<&InputArc> for Arc<Mutex<Input>> {
	fn from(input: &InputArc) -> Self { input.inner.clone() }
}

impl TryFrom<&AnyUserData> for InputArc {
	type Error = mlua::Error;

	fn try_from(value: &AnyUserData) -> Result<Self, Self::Error> {
		Ok(value.borrow::<Self>()?.clone())
	}
}

impl Spatial for InputArc {
	fn area(&self) -> Area { self.inner.lock().pos.into() }

	fn set_area(&mut self, area: Area) {
		self.inner.lock().repos(match area {
			Area::Rect(rect) => Position {
				origin: Origin::TopLeft,
				offset: Offset {
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

impl Widget for &InputArc {
	fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
	where
		Self: Sized,
	{
		self.inner.lock().render(rect, buf)
	}
}

impl UserData for InputArc {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		impl_area_method!(methods);

		methods.add_function("focus", |lua, (ud, focus): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.focus = focus;
			ud.into_lua(lua)
		});
	}
}
