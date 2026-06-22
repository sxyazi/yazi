use anyhow::Result;
use yazi_actor::Ctx;
use yazi_config::{KEYMAP, keymap::{Chord, Key}};
use yazi_core::which::WhichOpt;
use yazi_macro::act;
use yazi_shared::Layer;
use yazi_term::event::KeyEvent;

use crate::{Dispatcher, app::App};

pub(super) struct Router<'a> {
	app: &'a mut App,
}

impl<'a> Router<'a> {
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	pub(super) fn route(&mut self, key: KeyEvent) -> Result<bool> {
		use Layer as L;

		let core = &mut self.app.core;
		if core.help.visible && core.help.r#type(key)? {
			return Ok(true);
		}

		if let Some(mut guard) = core.input.lock_mut()
			&& guard.r#type(key)?
		{
			return Ok(true);
		}

		let layer = core.layer();
		let key = Key::from(key);
		Ok(match layer {
			L::Null | L::App | L::Notify => unreachable!(),
			L::Mgr | L::Tasks | L::Spot | L::Pick | L::Input | L::Confirm => {
				self.matches(layer, layer, key)
			}
			L::Help => self.matches(L::Help, L::Help, key) || self.matches(L::Input, L::Help, key),
			L::Cmp => self.matches(L::Cmp, L::Cmp, key) || self.matches(L::Input, L::Cmp, key),
			L::Which => core.which.r#type(key),
		})
	}

	fn matches(&mut self, src: Layer, dist: Layer, key: Key) -> bool {
		for chord in &*KEYMAP.chords(src) {
			let Chord { on, .. } = chord.as_ref();
			if on.is_empty() || on[0] != key {
				continue;
			}

			if on.len() > 1 {
				let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
				act!(which:activate, cx, WhichOpt::new(src, dist, key)).ok();
			} else {
				Dispatcher::new(self.app).dispatch_seq(chord.to_seq(dist));
			}
			return true;
		}
		false
	}
}
