use std::sync::atomic::{AtomicBool, Ordering};

use mlua::{Lua, Table, UserData};
use ratatui::layout::Rect;
use yazi_adaptor::ADAPTOR;

use super::{RectRef, Renderable};

pub static COLLISION: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Default)]
pub struct Clear {
	pub area: ratatui::layout::Rect,
}

impl Clear {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, area): (Table, RectRef)| Ok(Clear { area: *area }))?;

		let clear = lua.create_table()?;
		clear.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.raw_set("Clear", clear)
	}
}

impl ratatui::widgets::Widget for Clear {
	fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
	where
		Self: Sized,
	{
		ratatui::widgets::Clear.render(area, buf);

		let Some(r) = ADAPTOR.shown_load().and_then(|r| overlap(&area, &r)) else {
			return;
		};

		ADAPTOR.image_erase(r).ok();
		COLLISION.store(true, Ordering::Relaxed);
		for y in area.top()..area.bottom() {
			for x in area.left()..area.right() {
				buf.get_mut(x, y).set_skip(true);
			}
		}
	}
}

impl Renderable for Clear {
	fn area(&self) -> ratatui::layout::Rect { self.area }

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		<Self as ratatui::widgets::Widget>::render(Default::default(), self.area, buf);
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(*self).render(buf); }
}

impl UserData for Clear {}

#[inline]
const fn is_overlapping(a: &Rect, b: &Rect) -> bool {
	a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn overlap(a: &Rect, b: &Rect) -> Option<Rect> {
	if !is_overlapping(a, b) {
		return None;
	}

	let x = a.x.max(b.x);
	let y = a.y.max(b.y);
	let width = (a.x + a.width).min(b.x + b.width) - x;
	let height = (a.y + a.height).min(b.y + b.height) - y;
	Some(Rect { x, y, width, height })
}
