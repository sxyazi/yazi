use ratatui_core::layout::{Position, Rect};
use yazi_shared::Layer;
use yazi_shim::ratatui::Padable;
use yazi_tty::sequence::SetCursorStyle;

use crate::{cmp::Cmp, confirm::Confirm, help::Help, input::{Input, InputGuard}, mgr::Mgr, notify::Notify, pick::Pick, tab::{Folder, Tab}, tasks::Tasks, which::Which};

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
		if let Some(cursor) = self.help.cursor() {
			let Rect { x, y, .. } = self.mgr.area(self.help.position).padding(self.help.padding());
			return Some((Position { x: x + cursor, y }, self.help.cursor_shape()?));
		}

		if let Some(guard) = self.input.lock() {
			let Rect { x, y, .. } = match &guard {
				InputGuard::Main(_) => self.mgr.area(self.input.position()?).padding(self.input.padding()),
				InputGuard::Alt(_) => self.mgr.area(self.input.position()?),
			};
			return Some((Position { x: x + guard.cursor(), y }, guard.cursor_shape()));
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
