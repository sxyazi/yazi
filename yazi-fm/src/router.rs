use yazi_config::{KEYMAP, keymap::{Chord, ChordCow, Key}};
use yazi_macro::emit;
use yazi_shared::Layer;

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
		let layer = cx.layer();

		if cx.help.visible && cx.help.r#type(&key) {
			return true;
		}
		if cx.input.visible && cx.input.r#type(&key) {
			return true;
		}

		use Layer as L;
		match layer {
			L::App => unreachable!(),
			L::Mgr | L::Tasks | L::Spot | L::Pick | L::Input | L::Confirm | L::Help => {
				self.matches(layer, key)
			}
			L::Cmp => self.matches(L::Cmp, key) || self.matches(L::Input, key),
			L::Which => cx.which.r#type(key),
		}
	}

	#[inline]
	fn matches(&mut self, layer: Layer, key: Key) -> bool {
		for chord @ Chord { on, .. } in KEYMAP.get(layer) {
			if on.is_empty() || on[0] != key {
				continue;
			}

			if on.len() > 1 {
				self.app.cx.which.show_with(key, layer);
			} else {
				emit!(Seq(ChordCow::from(chord).into_seq()));
			}
			return true;
		}
		false
	}
}
