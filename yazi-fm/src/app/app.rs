use std::sync::atomic::Ordering;

use anyhow::{Ok, Result};
use crossterm::event::KeyEvent;
use ratatui::{backend::Backend, prelude::Rect};
use yazi_config::{keymap::Key, BOOT};
use yazi_core::{input::InputMode, preview::COLLISION, Ctx};
use yazi_shared::{emit, event::{Event, Exec}, files::FilesOp, term::Term, Layer};

use crate::{Executor, Logs, Panic, Root, Signals};

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
		let mut app = Self { cx: Ctx::make(), term: Some(term), signals };

		while let Some(event) = app.signals.recv().await {
			match event {
				Event::Quit(no_cwd_file) => {
					app.dispatch_quit(no_cwd_file);
					break;
				}
				Event::Key(key) => app.dispatch_key(key),
				Event::Paste(str) => app.dispatch_paste(str),
				Event::Render(_) => app.dispatch_render()?,
				Event::Resize(cols, rows) => app.dispatch_resize(cols, rows),
				Event::Call(exec, layer) => app.dispatch_call(exec, layer),
				event => app.dispatch_module(event),
			}
		}
		Ok(())
	}

	fn dispatch_quit(&mut self, no_cwd_file: bool) {
		if let Some(p) = BOOT.cwd_file.as_ref().filter(|_| !no_cwd_file) {
			let cwd = self.cx.manager.cwd().as_os_str();
			std::fs::write(p, cwd.as_encoded_bytes()).ok();
		}
		Term::goodbye(|| false);
	}

	fn dispatch_key(&mut self, key: KeyEvent) {
		let key = Key::from(key);
		if Executor::new(self).handle(key) {
			emit!(Render);
		}
	}

	fn dispatch_paste(&mut self, str: String) {
		if self.cx.input.visible {
			let input = &mut self.cx.input;
			if input.mode() == InputMode::Insert && input.type_str(&str) {
				emit!(Render);
			}
		}
	}

	fn dispatch_render(&mut self) -> Result<()> {
		let Some(term) = &mut self.term else {
			return Ok(());
		};

		let collision = COLLISION.swap(false, Ordering::Relaxed);
		let frame = term.draw(|f| {
			yazi_plugin::scope(&self.cx, |_| {
				f.render_widget(Root::new(&self.cx), f.size());
			});

			if let Some((x, y)) = self.cx.cursor() {
				f.set_cursor(x, y);
			}
		})?;
		if !COLLISION.load(Ordering::Relaxed) {
			if collision {
				// Reload preview if collision is resolved
				self.cx.manager.active_mut().preview.reset_image();
				self.cx.manager.peek(0);
			}
			return Ok(());
		}

		let mut patches = Vec::new();
		for x in frame.area.left()..frame.area.right() {
			for y in frame.area.top()..frame.area.bottom() {
				let cell = frame.buffer.get(x, y);
				if cell.skip {
					patches.push((x, y, cell.clone()));
				}
			}
		}

		term.backend_mut().draw(patches.iter().map(|(x, y, cell)| (*x, *y, cell)))?;
		if let Some((x, y)) = self.cx.cursor() {
			term.show_cursor()?;
			term.set_cursor(x, y)?;
		}
		term.backend_mut().flush()?;
		Ok(())
	}

	fn dispatch_resize(&mut self, cols: u16, rows: u16) {
		if let Some(term) = &mut self.term {
			term.resize(Rect::new(0, 0, cols, rows)).ok();
		}

		self.cx.manager.current_mut().set_page(true);
		self.cx.manager.active_mut().preview.reset();
		self.cx.manager.peek(0);
		emit!(Render);
	}

	#[inline]
	fn dispatch_call(&mut self, exec: Vec<Exec>, layer: Layer) {
		if Executor::new(self).dispatch(&exec, layer) {
			emit!(Render);
		}
	}

	fn dispatch_module(&mut self, event: Event) {
		let manager = &mut self.cx.manager;
		let tasks = &mut self.cx.tasks;
		match event {
			Event::Files(op) => {
				let calc = !matches!(op, FilesOp::Size(..) | FilesOp::IOErr(_));
				let b = match op {
					FilesOp::IOErr(..) => manager.update_ioerr(op),
					_ => manager.update_read(op),
				};
				if b {
					emit!(Render);
				}
				if calc {
					tasks.precache_size(&manager.current().files);
				}
			}
			Event::Pages(page) => {
				let targets = self.cx.manager.current().paginate(page);
				tasks.precache_mime(targets, &self.cx.manager.mimetype);
			}
			Event::Mimetype(mimes) => {
				if manager.update_mimetype(mimes, tasks) {
					emit!(Render);
					manager.peek(0);
				}
			}
			Event::Preview(lock) => {
				if manager.active_mut().update_preview(lock) {
					emit!(Render);
				}
			}
			_ => unreachable!(),
		}
	}
}
