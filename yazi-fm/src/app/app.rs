use std::sync::atomic::Ordering;

use anyhow::Result;
use crossterm::event::KeyEvent;
use yazi_config::keymap::Key;
use yazi_core::input::InputMode;
use yazi_macro::emit;
use yazi_shared::{Layer, event::{CmdCow, Event, NEED_RENDER}};

use crate::{Ctx, Executor, Router, Signals, Term, lives::Lives};

pub(crate) struct App {
	pub(crate) cx:      Ctx,
	pub(crate) term:    Option<Term>,
	pub(crate) signals: Signals,
}

impl App {
	pub(crate) async fn serve() -> Result<()> {
		let term = Term::start()?;
		let (mut rx, signals) = (Event::take(), Signals::start()?);

		Lives::register()?;
		let mut app = Self { cx: Ctx::make(), term: Some(term), signals };
		app.render();

		let mut times = 0;
		let mut events = Vec::with_capacity(200);
		while rx.recv_many(&mut events, 50).await > 0 {
			for event in events.drain(..) {
				times += 1;
				app.dispatch(event)?;
			}

			if !NEED_RENDER.swap(false, Ordering::Relaxed) {
				continue;
			}

			if times >= 50 {
				times = 0;
				app.render();
			} else if let Ok(event) = rx.try_recv() {
				events.push(event);
				emit!(Render);
			} else {
				times = 0;
				app.render();
			}
		}
		Ok(())
	}

	#[inline]
	fn dispatch(&mut self, event: Event) -> Result<()> {
		match event {
			Event::Call(cmd, layer) => self.dispatch_call(cmd, layer),
			Event::Seq(cmds, layer) => self.dispatch_seq(cmds, layer),
			Event::Render => self.dispatch_render(),
			Event::Key(key) => self.dispatch_key(key),
			Event::Mouse(mouse) => self.mouse(mouse),
			Event::Resize => self.resize(()),
			Event::Paste(str) => self.dispatch_paste(str),
			Event::Quit(opt) => self.quit(opt),
		}
		Ok(())
	}

	#[inline]
	fn dispatch_call(&mut self, cmd: CmdCow, layer: Layer) {
		Executor::new(self).execute(cmd, layer);
	}

	#[inline]
	fn dispatch_seq(&mut self, mut cmds: Vec<CmdCow>, layer: Layer) {
		if let Some(last) = cmds.pop() {
			Executor::new(self).execute(last, layer);
		}
		if !cmds.is_empty() {
			emit!(Seq(cmds, layer));
		}
	}

	#[inline]
	fn dispatch_render(&mut self) { NEED_RENDER.store(true, Ordering::Relaxed); }

	#[inline]
	fn dispatch_key(&mut self, key: KeyEvent) { Router::new(self).route(Key::from(key)); }

	#[inline]
	fn dispatch_paste(&mut self, str: String) {
		if self.cx.input.visible {
			let input = &mut self.cx.input;
			if input.mode() == InputMode::Insert {
				input.type_str(&str);
			}
		}
	}
}
