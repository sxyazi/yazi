use mlua::{ObjectLike, Table};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_plugin::{LUA, elements::render_once};

use super::{cmp, confirm, help, input, mgr, pick, spot, tasks, which};
use crate::Ctx;

pub(super) struct Root<'a> {
	cx: &'a Ctx,
}

impl<'a> Root<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }

	pub(super) fn reflow(area: Rect) -> mlua::Result<Table> {
		let area = yazi_plugin::elements::Rect::from(area);
		let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;
		root.call_method("reflow", ())
	}
}

impl Widget for Root<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let mut f = || {
			let area = yazi_plugin::elements::Rect::from(area);
			let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;

			render_once(root.call_method("redraw", ())?, buf, |p| self.cx.mgr.area(p));
			Ok::<_, mlua::Error>(())
		};
		if let Err(e) = f() {
			error!("Failed to redraw the `Root` component:\n{e}");
		}

		mgr::Preview::new(self.cx).render(area, buf);
		mgr::Modal::new(self.cx).render(area, buf);

		if self.cx.tasks.visible {
			tasks::Tasks::new(self.cx).render(area, buf);
		}

		if self.cx.active().spot.visible() {
			spot::Spot::new(self.cx).render(area, buf);
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
			help::Help::new(self.cx).render(area, buf);
		}

		if self.cx.cmp.visible {
			cmp::Cmp::new(self.cx).render(area, buf);
		}

		if self.cx.which.visible {
			which::Which::new(self.cx).render(area, buf);
		}
	}
}
