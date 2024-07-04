use std::mem;

use mlua::{AnyUserData, Table, TableExt};
use tracing::error;
use yazi_plugin::{cast_to_renderable, LUA};

pub(crate) struct Progress;

impl Progress {
	pub(crate) fn partial_render(
		buf: &mut ratatui::buffer::Buffer,
	) -> Vec<Vec<(u16, u16, ratatui::buffer::Cell)>> {
		let mut patches = vec![];
		let mut f = || {
			let comp: Table = LUA.globals().raw_get("Progress")?;
			for widget in comp.call_method::<_, Vec<AnyUserData>>("partial_render", ())? {
				let Some(w) = cast_to_renderable(widget) else { continue };

				let area = w.area();
				w.render(buf);

				let mut patch = Vec::with_capacity(area.width as usize * area.height as usize);
				for y in area.top()..area.bottom() {
					for x in area.left()..area.right() {
						patch.push((x, y, mem::take(buf.get_mut(x, y))));
					}
				}

				buf.reset();
				patches.push(patch);
			}

			Ok::<_, anyhow::Error>(())
		};

		if let Err(e) = f() {
			error!("{e}");
		}
		patches
	}
}
