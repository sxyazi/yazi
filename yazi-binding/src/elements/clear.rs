use std::sync::atomic::{AtomicBool, Ordering};

use mlua::{IntoLua, Lua, MetaMethod, Table, UserData, Value};
use yazi_adapter::ADAPTOR;

use super::Area;

pub static COLLISION: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Default)]
pub struct Clear {
	pub area: Area,
}

impl Clear {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, area): (Table, Area)| Ok(Clear { area }))?;

		let clear = lua.create_table()?;
		clear.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		clear.into_lua(lua)
	}

	pub(super) fn render(
		self,
		buf: &mut ratatui::buffer::Buffer,
		trans: impl FnOnce(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) {
		<Self as ratatui::widgets::Widget>::render(Default::default(), self.area.transform(trans), buf);
	}
}

impl ratatui::widgets::Widget for Clear {
	fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::prelude::Buffer)
	where
		Self: Sized,
	{
		ratatui::widgets::Clear.render(area, buf);

		let Some(r) = ADAPTOR.get().shown_load().and_then(|r| overlap(area, r)) else {
			return;
		};

		ADAPTOR.get().image_erase(r).ok();
		COLLISION.store(true, Ordering::Relaxed);
		for y in r.top()..r.bottom() {
			for x in r.left()..r.right() {
				buf[(x, y)].set_skip(true);
			}
		}
	}
}

impl UserData for Clear {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
	}
}

#[inline]
const fn is_overlapping(a: ratatui::layout::Rect, b: ratatui::layout::Rect) -> bool {
	a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn overlap(a: ratatui::layout::Rect, b: ratatui::layout::Rect) -> Option<ratatui::layout::Rect> {
	if !is_overlapping(a, b) {
		return None;
	}

	let x = a.x.max(b.x);
	let y = a.y.max(b.y);
	let width = (a.x + a.width).min(b.x + b.width) - x;
	let height = (a.y + a.height).min(b.y + b.height) - y;
	Some(ratatui::layout::Rect { x, y, width, height })
}
