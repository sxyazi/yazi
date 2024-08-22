use mlua::{Table, TableExt};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_plugin::{bindings::Cast, elements::render_widgets, LUA};

use super::{completion, confirm, input, select, tasks, which};
use crate::{components, help, Ctx};

pub(super) struct Root<'a> {
	cx: &'a Ctx,
}

impl<'a> Root<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Root<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let mut f = || {
			let area = yazi_plugin::elements::Rect::cast(&LUA, area)?;
			let root = LUA.globals().raw_get::<_, Table>("Root")?.call_method::<_, Table>("new", area)?;

			render_widgets(root.call_method("render", ())?, buf);
			Ok::<_, mlua::Error>(())
		};
		if let Err(e) = f() {
			error!("Failed to render the `Root` component:\n{e}");
		}

		components::Preview::new(self.cx).render(area, buf);

		if self.cx.tasks.visible {
			tasks::Layout::new(self.cx).render(area, buf);
		}

		if self.cx.select.visible {
			select::Select::new(self.cx).render(area, buf);
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
