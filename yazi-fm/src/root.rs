use mlua::{ObjectLike, Table};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_binding::elements::render_once;
use yazi_config::THEME;
use yazi_core::Core;
use yazi_plugin::LUA;

use super::{cmp, confirm, help, input, mgr, pick, spot, tasks, which};

pub(super) struct Root<'a> {
	core: &'a Core,
}

impl<'a> Root<'a> {
	pub(super) fn new(core: &'a Core) -> Self { Self { core } }

	pub(super) fn reflow(area: Rect) -> mlua::Result<Table> {
		let area = yazi_binding::elements::Rect::from(area);
		let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;
		root.call_method("reflow", ())
	}
}

impl Widget for Root<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let mut f = || {
			let area = yazi_binding::elements::Rect::from(area);
			let root = LUA.globals().raw_get::<Table>("Root")?.call_method::<Table>("new", area)?;

			render_once(root.call_method("redraw", ())?, buf, |p| self.core.mgr.area(p));
			Ok::<_, mlua::Error>(())
		};
		if let Err(e) = f() {
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

		if self.core.which.visible {
			which::Which::new(self.core).render(area, buf);
		}

		// Fill background on all cells to create an opaque background (like fzf does).
		// This is done AFTER all rendering so text/foreground colors are already set.
		if !THEME.app.background.is_empty() {
			if let Ok(bg_color) = THEME.app.background.parse::<ratatui::style::Color>() {
				for y in area.top()..area.bottom() {
					for x in area.left()..area.right() {
						// Set background on every cell, but don't touch foreground
						buf[(x, y)].set_bg(bg_color);
					}
				}
			}
		}
	}
}
