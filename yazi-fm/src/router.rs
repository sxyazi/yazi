use crossterm::event::KeyCode;
use yazi_config::{KEYMAP, keymap::{Chord, Key}};
use yazi_shared::{Layer, emit};

use crate::app::App;

pub(super) struct Router<'a> {
	app: &'a mut App,
}

impl<'a> Router<'a> {
	#[inline]
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	#[inline]
	pub(super) fn route(&mut self, key: Key) -> bool {
		let cx = &mut self.app.cx;

		if cx.which.visible {
			return cx.which.type_(key);
		}
		if cx.help.visible && cx.help.type_(&key) {
			return true;
		}
		if cx.input.visible && cx.input.type_(&key) {
			return true;
		}

		if cx.completion.visible {
			self.matches(Layer::Completion, key) || self.matches(Layer::Input, key)
		} else if cx.help.visible {
			self.matches(Layer::Help, key)
		} else if cx.input.visible {
			self.matches(Layer::Input, key)
		} else if cx.confirm.visible {
			self.matches(Layer::Confirm, key)
		} else if cx.select.visible {
			self.matches(Layer::Select, key)
		} else if cx.tasks.visible {
			self.matches(Layer::Tasks, key)
		} else {
			self.matches(Layer::Manager, key)
		}
	}

	#[inline]
	fn matches(&mut self, layer: Layer, key: Key) -> bool {
		for ctrl @ Chord { on, .. } in KEYMAP.get(layer) {
			if on.is_empty() || on[0] != key {
				continue;
			}

			if on.len() > 1 {
				//for `copy` hotkeys starting with 'c', we need to escape visual mode first so that the `Which` UI can check how many files
				// are selected to decide using plural or singular on command names.
				if key.code == KeyCode::Char('c') {
					if !self.app.cx.manager.active_mut().try_escape_visual() {
						return false;
					}
				}
				self.app.cx.which.show_with(key, layer);
			} else {
				emit!(Seq(ctrl.to_seq(), layer));
			}
			return true;
		}
		false
	}
}
