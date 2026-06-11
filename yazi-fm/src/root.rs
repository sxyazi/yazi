use mlua::{ObjectLike, Table};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_core::Core;
use yazi_plugin::LUA;

use super::{cmp, confirm, help, input, mgr, pick, spot, tasks, which};
use crate::Renderer;

pub(super) struct Root<'a> {
	core: &'a mut Core,
}

impl<'a> Root<'a> {
	pub(super) fn new(core: &'a mut Core) -> Self { Self { core } }

	pub(super) fn reflow(area: Rect) -> mlua::Result<Table> {
		let area = yazi_binding::elements::Rect::from(area);
		let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;
		root.call_method("reflow", ())
	}
}

impl Widget for Root<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		if let Err(e) = Renderer::new(self.core, "Root").render(area, buf) {
			error!("Failed to redraw the `Root` component:\n{e}");
		}

		mgr::Preview::new(self.core).render(area, buf);
		mgr::Modal::new(self.core).render(area, buf);

		if self.core.tasks.visible {
			tasks::Tasks::new(self.core).render(area, buf);
		}

		if self.core.active().spot.visible() {
			spot::Spot::new(self.core).render(area, buf);
		}

		if self.core.pick.visible {
			pick::Pick::new(self.core).render(area, buf);
		}

		if self.core.input.visible {
			input::Input::new(self.core).render(area, buf);
		}

		if self.core.confirm.visible {
			confirm::Confirm::new(self.core).render(area, buf);
		}

		if self.core.help.visible {
			help::Help::new(self.core).render(area, buf);
		}

		if self.core.cmp.visible {
			cmp::Cmp::new(self.core).render(area, buf);
		}

		if self.core.which.active {
			which::Which::new(self.core).render(area, buf);
		}
	}
}
