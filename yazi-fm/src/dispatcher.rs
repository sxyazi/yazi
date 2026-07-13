use std::sync::atomic::Ordering;

use anyhow::Result;
use tracing::warn;
use yazi_actor::Ctx;
use yazi_macro::{act, emit};
use yazi_shared::event::{ActionCow, Event, NEED_RENDER};
use yazi_term::event::{ClipboardEvent, DndEvent, Event as TermEvent, KeyEvent, MouseEvent};
use yazi_widgets::input::InputMode;

use crate::{Executor, Router, app::App};

pub(super) struct Dispatcher<'a> {
	app: &'a mut App,
}

impl<'a> Dispatcher<'a> {
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	pub(super) fn dispatch(&mut self, event: Event) {
		let result = match event {
			Event::Call(action) => Ok(self.dispatch_call(action)),
			Event::Seq(actions) => Ok(self.dispatch_seq(actions)),
			Event::Render(partial) => self.dispatch_render(partial),
			Event::Term(TermEvent::Key(key)) => self.dispatch_key(key),
			Event::Term(TermEvent::Mouse(mouse)) => self.dispatch_mouse(mouse),
			Event::Term(TermEvent::Resize(_)) => self.dispatch_resize(),
			Event::Term(TermEvent::FocusIn) => self.dispatch_focus(),
			Event::Term(TermEvent::FocusOut) => Ok(()),
			Event::Term(TermEvent::Paste(str)) => self.dispatch_paste(str),
			Event::Term(TermEvent::Dnd(dnd)) => self.dispatch_dnd(dnd),
			Event::Term(TermEvent::Clipboard(clip)) => self.dispatch_clipboard(clip),
		};

		if let Err(e) = &result {
			warn!("Event dispatch error: {e:?}");
		}
	}

	fn dispatch_call(&mut self, action: ActionCow) {
		let tx = action.replier().cloned();
		let result = Executor::new(self.app).execute(action);

		if let Err(e) = &result {
			warn!("Call dispatch error: {e:?}");
		}
		if let Some(tx) = tx {
			tx.send(result).ok();
		}
	}

	pub(super) fn dispatch_seq(&mut self, mut actions: Vec<ActionCow>) {
		if let Some(last) = actions.pop() {
			self.dispatch_call(last);
		}
		if !actions.is_empty() {
			emit!(Seq(actions));
		}
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
		Router::new(self.app).route(key)?;
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
		if let Some(mut guard) = self.app.core.input.lock_mut() {
			if guard.mode() == InputMode::Insert {
				guard.type_str(&str)?;
			} else if guard.mode() == InputMode::Replace {
				guard.replace_str(&str)?;
			}
		}
		Ok(())
	}

	fn dispatch_dnd(&mut self, dnd: DndEvent) -> Result<()> {
		let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
		act!(app:dnd, cx, dnd).map(|_| ())
	}

	fn dispatch_clipboard(&mut self, clip: ClipboardEvent) -> Result<()> {
		if self.app.core.input.main.visible && clip.is_read() {
			if let Some(text) = clip.text() {
				self.dispatch_paste(text)?;
			}
			Ok(())
		} else {
			let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
			act!(app:clipboard, cx, clip).map(|_| ())
		}
	}
}
