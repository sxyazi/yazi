use std::sync::atomic::Ordering;

use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use tracing::warn;
use yazi_actor::Ctx;
use yazi_config::keymap::Key;
use yazi_macro::{act, emit};
use yazi_shared::event::{ActionCow, Event, NEED_RENDER};
use yazi_widgets::input::InputMode;

use crate::{Executor, Router, app::App};

pub(super) struct Dispatcher<'a> {
	app: &'a mut App,
}

impl<'a> Dispatcher<'a> {
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	pub(super) fn dispatch(&mut self, event: Event) {
		let result = match event {
			Event::Call(action) => self.dispatch_call(action),
			Event::Seq(actions) => self.dispatch_seq(actions),
			Event::Render(partial) => self.dispatch_render(partial),
			Event::Key(key) => self.dispatch_key(key),
			Event::Mouse(mouse) => self.dispatch_mouse(mouse),
			Event::Resize => self.dispatch_resize(),
			Event::Focus => self.dispatch_focus(),
			Event::Paste(str) => self.dispatch_paste(str),
		};

		if let Err(e) = &result {
			warn!("Event dispatch error: {e:?}");
		}
	}

	fn dispatch_call(&mut self, action: ActionCow) -> Result<()> {
		let tx = action.replier().cloned();
		let result = Executor::new(self.app).execute(action);

		if let Err(e) = &result {
			warn!("Call dispatch error: {e:?}");
		}
		if let Some(tx) = tx {
			tx.send(result).ok();
		}
		Ok(())
	}

	fn dispatch_seq(&mut self, mut actions: Vec<ActionCow>) -> Result<()> {
		if let Some(last) = actions.pop() {
			self.dispatch_call(last)?;
		}
		if !actions.is_empty() {
			emit!(Seq(actions));
		}
		Ok(())
	}

	fn dispatch_render(&mut self, partial: bool) -> Result<()> {
		if partial {
			_ = NEED_RENDER.compare_exchange(0, 2, Ordering::Relaxed, Ordering::Relaxed);
		} else {
			NEED_RENDER.store(1, Ordering::Relaxed);
		}
		Ok(())
	}

	fn dispatch_key(&mut self, key: KeyEvent) -> Result<()> {
		Router::new(self.app).route(Key::from(key))?;
		Ok(())
	}

	fn dispatch_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
		let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
		act!(app:mouse, cx, mouse).map(|_| ())
	}

	fn dispatch_resize(&mut self) -> Result<()> {
		let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
		act!(app:resize, cx, crate::Root::reflow as fn(_) -> _).map(|_| ())
	}

	fn dispatch_focus(&mut self) -> Result<()> {
		let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
		act!(app:focus, cx).map(|_| ())
	}

	fn dispatch_paste(&mut self, str: String) -> Result<()> {
		if self.app.core.input.visible {
			let input = &mut self.app.core.input;
			if input.mode() == InputMode::Insert {
				input.type_str(&str)?;
			} else if input.mode() == InputMode::Replace {
				input.replace_str(&str)?;
			}
		}
		Ok(())
	}
}
