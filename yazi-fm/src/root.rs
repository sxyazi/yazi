use mlua::{Table, TableExt};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_plugin::{LUA, elements::render_widgets};

use super::{completion, confirm, input, pick, tasks, which};
use crate::{Ctx, components, help};

pub(super) struct Root<'a> {
	cx: &'a Ctx,
}

impl<'a> Root<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }

	pub(super) fn reflow<'lua>(area: Rect) -> mlua::Result<Table<'lua>> {
		let area = yazi_plugin::elements::Rect::from(area);
		let root = LUA.globals().raw_get::<_, Table>("Root")?.call_method::<_, Table>("new", area)?;
		root.call_method("reflow", ())
	}
}

impl<'a> Widget for Root<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let mut f = || {
			let area = yazi_plugin::elements::Rect::from(area);
			let root = LUA.globals().raw_get::<_, Table>("Root")?.call_method::<_, Table>("new", area)?;

			render_widgets(root.call_method("redraw", ())?, buf);
			Ok::<_, mlua::Error>(())
		};
		if let Err(e) = f() {
			error!("Failed to redraw the `Root` component:\n{e}");
		}

		components::Preview::new(self.cx).render(area, buf);

		if self.cx.tasks.visible {
			tasks::Layout::new(self.cx).render(area, buf);
		}

		if self.cx.pick.visible {
			pick::Pick::new(self.cx).render(area, buf);
		}

		if self.cx.input.visible {
			input::Input::new(self.cx).render(area, buf);
		}

		if self.cx.confirm.visible {
			confirm::Confirm::new(self.cx).render(area, buf);
		}

		if self.cx.help.visible {
			help::Layout::new(self.cx).render(area, buf);
		}

		if self.cx.completion.visible {
			completion::Completion::new(self.cx).render(area, buf);
		}

		if self.cx.which.visible {
			which::Which::new(self.cx).render(area, buf);
		}
	}
}
