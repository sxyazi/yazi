use mlua::{ObjectLike, Table};
use ratatui::{buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, widgets::Widget};
use tracing::error;
use yazi_binding::elements::render_once;
use yazi_config::{THEME, YAZI};
use yazi_core::Core;
use yazi_plugin::LUA;

use super::{bg_render::*, cmp, confirm, help, input, mgr, pick, spot, tasks, which};

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

		// Calculate the Tab area (excluding Header, Tabs, and Status)
		// This matches the Root layout in root.lua
		let tabs_height = if self.core.mgr.tabs.len() > 1 { 1 } else { 0 };
		let root_chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Length(1),           // Header
				Constraint::Length(tabs_height), // Tabs
				Constraint::Fill(1),             // Tab content (the 3 panes)
				Constraint::Length(1),           // Status
			])
			.split(area);
		let tab_area = root_chunks[2];

		// Apply per-pane backgrounds first, then app-wide background
		// Calculate pane areas using the manager ratio within the tab area
		let ratio = YAZI.mgr.ratio.get();
		let chunks = Layout::default()
			.direction(Direction::Horizontal)
			.constraints([
				Constraint::Ratio(ratio.parent as u32, ratio.all as u32),
				Constraint::Ratio(ratio.current as u32, ratio.all as u32),
				Constraint::Ratio(ratio.preview as u32, ratio.all as u32),
			])
			.split(tab_area);

		// Apply pane backgrounds (if configured)
		// Skip borders: top/bottom rows and left/right edges where borders are drawn
		let parent_bg = THEME.app.parent_bg();
		if !parent_bg.is_empty() {
			if let Ok(bg_color) = parent_bg.parse::<ratatui::style::Color>() {
				apply_pane_bg_with_borders(buf, chunks[0], bg_color);
			}
		}

		let current_bg = THEME.app.current_bg();
		if !current_bg.is_empty() {
			if let Ok(bg_color) = current_bg.parse::<ratatui::style::Color>() {
				apply_pane_bg_no_vertical_borders(buf, chunks[1], bg_color);
			}
		}

		let preview_bg = THEME.app.preview_bg();
		if !preview_bg.is_empty() {
			if let Ok(bg_color) = preview_bg.parse::<ratatui::style::Color>() {
				apply_pane_bg_with_borders(buf, chunks[2], bg_color);
			}
		}

		// Fill background on all cells to create an opaque background (like fzf does).
		// This is done AFTER all rendering so text/foreground colors are already set.
		let bg_color_str = THEME.app.bg_color();
		if !bg_color_str.is_empty() {
			if let Ok(bg_color) = bg_color_str.parse::<ratatui::style::Color>() {
				apply_overall_bg(buf, area, bg_color);
			}
		}
	}
}
