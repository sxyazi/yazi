use crossterm::cursor::SetCursorStyle;
use ratatui::layout::{Position, Rect};
use yazi_shared::Layer;

use crate::{cmp::Cmp, confirm::Confirm, help::Help, input::Input, mgr::Mgr, notify::Notify, pick::Pick, tab::{Folder, Tab}, tasks::Tasks, which::Which};

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
}

impl Core {
	pub fn make() -> Self {
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
		}
	}

	pub fn cursor(&self) -> Option<(Position, SetCursorStyle)> {
		if self.input.visible {
			let Rect { x, y, .. } = self.mgr.area(self.input.position);
			return Some((
				Position { x: x + 1 + self.input.cursor(), y: y + 1 },
				self.input.cursor_shape(),
			));
		}
		if let Some((x, y)) = self.help.cursor() {
			return Some((Position { x, y }, self.help.cursor_shape()));
		}
		None
	}

	pub fn layer(&self) -> Layer {
		if self.which.visible {
			Layer::Which
		} else if self.cmp.visible {
			Layer::Cmp
		} else if self.help.visible {
			Layer::Help
		} else if self.confirm.visible {
			Layer::Confirm
		} else if self.input.visible {
			Layer::Input
		} else if self.pick.visible {
			Layer::Pick
		} else if self.active().spot.visible() {
			Layer::Spot
		} else if self.tasks.visible {
			Layer::Tasks
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

	#[inline]
	pub fn current_mut(&mut self) -> &mut Folder { self.mgr.current_mut() }

	#[inline]
	pub fn parent_mut(&mut self) -> Option<&mut Folder> { self.mgr.parent_mut() }
}
