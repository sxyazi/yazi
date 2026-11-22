use ratatui_core::layout::{Position, Rect};
use yazi_config::LAYOUT;
use yazi_shared::Layer;
use yazi_shim::ratatui::Padable;
use yazi_tty::sequence::SetCursorStyle;
use yazi_tui::Raterm;

use crate::{cmp::Cmp, confirm::Confirm, help::Help, input::{Input, InputGuard}, mgr::Mgr, notify::Notify, pick::Pick, tab::Tab, tasks::Tasks, which::Which};

pub struct Core {
	pub mgr:     Mgr,
	pub tasks:   Tasks,
	pub pick:    Pick,
	pub input:   Input,
	pub confirm: Confirm,
	pub help:    Help,
	pub cmp:     Cmp,
	pub which:   Which,
	pub notify:  Notify,
	pub term:    Option<Raterm>,
}

impl Core {
	pub fn make(term: Raterm) -> Self {
		Self {
			mgr:     Mgr::make(),
			tasks:   Tasks::serve(),
			pick:    Default::default(),
			input:   Default::default(),
			confirm: Default::default(),
			help:    Default::default(),
			cmp:     Default::default(),
			which:   Default::default(),
			notify:  Default::default(),
			term:    Some(term),
		}
	}

	pub fn cursor(&self) -> Option<(Position, SetCursorStyle)> {
		if let Some(cursor) = self.help.cursor() {
			let Rect { x, y, .. } = self.mgr.area(self.help.position).padding(self.help.padding());
			return Some((Position { x: x + cursor, y }, self.help.cursor_shape()?));
		}

		if let Some(guard) = self.input.lock() {
			let area = match &guard {
				InputGuard::Main(_) => self.input.main_field_area(&self.mgr),
				InputGuard::Alt(_) => self.mgr.area(self.input.position()?),
			};
			if self.input.main.visible && self.input.main.is_view_command() {
				if area.is_empty() {
					return None;
				}
				let x = area.x + guard.cursor().min(area.width - 1);
				return Some((Position { x, y: area.y }, guard.cursor_shape()));
			}
			return Some((Position { x: area.x + guard.cursor(), y: area.y }, guard.cursor_shape()));
		}

		if self.layer() == Layer::View
			&& let Some(view) = &self.active().view
		{
			let shape = if view.insert { SetCursorStyle::SteadyBar } else { SetCursorStyle::SteadyBlock };
			return Some((view.screen_cursor(LAYOUT.get().view)?, shape));
		}

		None
	}

	pub fn layer(&self) -> Layer {
		if self.which.active {
			Layer::Which
		} else if self.cmp.visible {
			Layer::Cmp
		} else if self.help.visible {
			Layer::Help
		} else if self.confirm.visible {
			Layer::Confirm
		} else if self.input.focus() {
			Layer::Input
		} else if self.pick.visible {
			Layer::Pick
		} else if self.active().spot.visible() {
			Layer::Spot
		} else if self.tasks.visible {
			Layer::Tasks
		} else if self.active().view.as_ref().is_some_and(|view| view.focused) {
			Layer::View
		} else {
			Layer::Mgr
		}
	}
}

impl Core {
	#[inline]
	pub fn active(&self) -> &Tab { self.mgr.active() }

	#[inline]
	pub fn active_mut(&mut self) -> &mut Tab { self.mgr.active_mut() }
}
