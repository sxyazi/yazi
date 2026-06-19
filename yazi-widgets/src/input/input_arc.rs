use std::{ops::Deref, sync::Arc};

use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use parking_lot::Mutex;
use ratatui_core::widgets::Widget;
use yazi_binding::{elements::{Area, Spatial}, impl_area_method};

use crate::input::{Input, InputOpt, InputStyles};

#[derive(Clone, Debug, Default)]
pub struct InputArc {
	area:  Area,
	inner: Arc<Mutex<Input>>,

	pub focus: bool,
}

impl Deref for InputArc {
	type Target = Arc<Mutex<Input>>;

	fn deref(&self) -> &Self::Target { &self.inner }
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
	fn area(&self) -> Area { self.area }

	fn set_area(&mut self, area: Area) { self.area = area; }
}

impl Widget for &InputArc {
	fn render(self, rect: ratatui_core::layout::Rect, buf: &mut ratatui_core::buffer::Buffer)
	where
		Self: Sized,
	{
		let mut guard = self.inner.lock();
		guard.repos(rect);
		guard.render(rect, buf)
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
