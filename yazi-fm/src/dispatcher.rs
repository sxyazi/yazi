use std::sync::atomic::Ordering;

use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use tracing::warn;
use yazi_actor::Ctx;
use yazi_config::keymap::Key;
use yazi_macro::{act, emit, succ};
use yazi_shared::{data::Data, event::{CmdCow, Event, NEED_RENDER}};
use yazi_widgets::input::InputMode;

use crate::{Executor, Router, app::App};

pub(super) struct Dispatcher<'a> {
	app: &'a mut App,
}

impl<'a> Dispatcher<'a> {
	#[inline]
	pub(super) fn new(app: &'a mut App) -> Self { Self { app } }

	#[inline]
	pub(super) fn dispatch(&mut self, event: Event) -> Result<()> {
		let result = match event {
			Event::Call(cmd) => self.dispatch_call(cmd),
			Event::Seq(cmds) => self.dispatch_seq(cmds),
			Event::Render(partial) => self.dispatch_render(partial),
			Event::Key(key) => self.dispatch_key(key),
			Event::Mouse(mouse) => self.dispatch_mouse(mouse),
			Event::Resize => self.dispatch_resize(),
			Event::Focus => self.dispatch_focus(),
			Event::Paste(str) => self.dispatch_paste(str),
		};

		if let Err(err) = result {
			warn!("Event dispatch error: {err:?}");
		}
		Ok(())
	}

	#[inline]
	fn dispatch_call(&mut self, cmd: CmdCow) -> Result<Data> { Executor::new(self.app).execute(cmd) }

	#[inline]
	fn dispatch_seq(&mut self, mut cmds: Vec<CmdCow>) -> Result<Data> {
		if let Some(last) = cmds.pop() {
			self.dispatch_call(last)?;
		}
		if !cmds.is_empty() {
			emit!(Seq(cmds));
		}
		succ!();
	}

	#[inline]
	fn dispatch_render(&mut self, partial: bool) -> Result<Data> {
		if partial {
			_ = NEED_RENDER.compare_exchange(0, 2, Ordering::Relaxed, Ordering::Relaxed);
		} else {
			NEED_RENDER.store(1, Ordering::Relaxed);
		}
		succ!()
	}

	#[inline]
	fn dispatch_key(&mut self, key: KeyEvent) -> Result<Data> {
		Router::new(self.app).route(Key::from(key))?;
		succ!();
	}

	#[inline]
	fn dispatch_mouse(&mut self, mouse: MouseEvent) -> Result<Data> {
		let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
		act!(app:mouse, cx, mouse)
	}

	#[inline]
	fn dispatch_resize(&mut self) -> Result<Data> {
		let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
		act!(app:resize, cx, crate::Root::reflow as fn(_) -> _)
	}

	#[inline]
	fn dispatch_focus(&mut self) -> Result<Data> {
		let cx = &mut Ctx::active(&mut self.app.core, &mut self.app.term);
		act!(app:focus, cx)
	}

	#[inline]
	fn dispatch_paste(&mut self, str: String) -> Result<Data> {
		if self.app.core.input.visible {
			let input = &mut self.app.core.input;
			if input.mode() == InputMode::Insert {
				input.type_str(&str)?;
			} else if input.mode() == InputMode::Replace {
				input.replace_str(&str)?;
			}
		}
		succ!();
	}
}
