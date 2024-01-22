use std::{collections::VecDeque, mem, sync::atomic::Ordering};

use anyhow::Result;
use crossterm::event::KeyEvent;
use yazi_config::keymap::Key;
use yazi_core::input::InputMode;
use yazi_shared::{emit, event::{Event, Exec, NEED_RENDER}, term::Term, Layer};

use crate::{lives::Lives, Ctx, Executor, Logs, Panic, Signals};

pub(crate) struct App {
	pub(crate) cx:      Ctx,
	pub(crate) term:    Option<Term>,
	pub(crate) signals: Signals,
}

impl App {
	pub(crate) async fn run() -> Result<()> {
		Panic::install();
		let _log = Logs::init()?;
		let term = Term::start()?;
		let signals = Signals::start()?;

		Lives::register()?;
		let mut app = Self { cx: Ctx::make(), term: Some(term), signals };
		app.render()?;

		let mut events = Vec::with_capacity(50);
		let mut render_in_place = false;
		while app.signals.rx.recv_many(&mut events, 50).await > 0 {
			for event in events.drain(..) {
				match event {
					Event::Call(exec, layer) => app.dispatch_call(exec, layer),
					Event::Seq(execs, layer) => app.dispatch_seq(execs, layer),
					Event::Render => render_in_place = true,
					Event::Key(key) => app.dispatch_key(key),
					Event::Resize => app.resize()?,
					Event::Paste(str) => app.dispatch_paste(str),
					Event::Quit(opt) => {
						app.quit(opt)?;
						return Ok(());
					}
				}
			}

			if mem::replace(&mut render_in_place, false) {
				if let Ok(event) = app.signals.rx.try_recv() {
					events.push(event);
					NEED_RENDER.store(true, Ordering::Relaxed);
				} else {
					app.render()?;
				}
			}

			if NEED_RENDER.swap(false, Ordering::Relaxed) {
				emit!(Render);
			}
		}
		Ok(())
	}

	#[inline]
	fn dispatch_key(&mut self, key: KeyEvent) { Executor::new(self).handle(Key::from(key)); }

	fn dispatch_paste(&mut self, str: String) {
		if self.cx.input.visible {
			let input = &mut self.cx.input;
			if input.mode() == InputMode::Insert {
				input.type_str(&str);
			}
		}
	}

	#[inline]
	fn dispatch_call(&mut self, exec: Exec, layer: Layer) {
		Executor::new(self).dispatch(exec, layer);
	}

	#[inline]
	fn dispatch_seq(&mut self, mut execs: VecDeque<Exec>, layer: Layer) {
		if let Some(exec) = execs.pop_front() {
			Executor::new(self).dispatch(exec, layer);
		}
		if !execs.is_empty() {
			emit!(Seq(execs, layer));
		}
	}
}
