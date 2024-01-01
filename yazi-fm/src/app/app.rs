use anyhow::{Ok, Result};
use crossterm::event::KeyEvent;
use yazi_config::{keymap::Key, ARGS};
use yazi_core::input::InputMode;
use yazi_shared::{event::{Event, Exec}, render, term::Term, Layer};

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
		while let Some(event) = app.signals.recv().await {
			match event {
				Event::Quit(no_cwd_file) => {
					app.dispatch_quit(no_cwd_file);
					break;
				}
				Event::Key(key) => app.dispatch_key(key),
				Event::Paste(str) => app.dispatch_paste(str),
				// TODO: render
				// Event::Render(_) => app.render()?,
				Event::Resize(cols, rows) => app.dispatch_resize(cols, rows)?,
				Event::Call(exec, layer) => app.dispatch_call(exec, layer),
				event => app.dispatch_module(event),
			}
		}
		Ok(())
	}

	fn dispatch_quit(&mut self, no_cwd_file: bool) {
		if let Some(p) = ARGS.cwd_file.as_ref().filter(|_| !no_cwd_file) {
			let cwd = self.cx.manager.cwd().as_os_str();
			std::fs::write(p, cwd.as_encoded_bytes()).ok();
		}
		Term::goodbye(|| false);
	}

	fn dispatch_key(&mut self, key: KeyEvent) { Executor::new(self).handle(Key::from(key)); }

	fn dispatch_paste(&mut self, str: String) {
		if self.cx.input.visible {
			let input = &mut self.cx.input;
			if input.mode() == InputMode::Insert && input.type_str(&str) {
				render!();
			}
		}
	}

	fn dispatch_resize(&mut self, _: u16, _: u16) -> Result<()> {
		self.cx.manager.active_mut().preview.reset();
		self.render()?;

		self.cx.manager.current_mut().set_page(true);
		self.cx.manager.peek(false);
		Ok(())
	}

	#[inline]
	fn dispatch_call(&mut self, exec: Vec<Exec>, layer: Layer) {
		Executor::new(self).dispatch(&exec, layer);
	}

	fn dispatch_module(&mut self, event: Event) {
		let tasks = &mut self.cx.tasks;
		match event {
			Event::Pages(page) => {
				let targets = self.cx.manager.current().paginate(page);
				tasks.preload_paged(targets, &self.cx.manager.mimetype);
			}
			_ => unreachable!(),
		}
	}
}
