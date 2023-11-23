use std::{ffi::OsString, sync::atomic::Ordering};

use anyhow::{Ok, Result};
use crossterm::event::KeyEvent;
use ratatui::{backend::Backend, prelude::Rect};
use tokio::sync::oneshot;
use yazi_config::{keymap::{Exec, Key, KeymapLayer}, BOOT};
use yazi_core::{emit, files::FilesOp, input::InputMode, manager::Manager, preview::COLLISION, Ctx, Event};
use yazi_shared::Term;

use crate::{Executor, Logs, Panic, Root, Signals};

pub(super) struct App {
	cx:      Ctx,
	term:    Option<Term>,
	signals: Signals,
}

impl App {
	pub(super) async fn run() -> Result<()> {
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
				Event::Stop(state, tx) => app.dispatch_stop(state, tx),
				Event::Call(exec, layer) => app.dispatch_call(exec, layer),
				event => app.dispatch_module(event),
			}
		}
		Ok(())
	}

	fn dispatch_quit(&mut self, no_cwd_file: bool) {
		if let Some(p) = BOOT.cwd_file.as_ref().filter(|_| !no_cwd_file) {
			let cwd = self.cx.manager.cwd().as_os_str();

			#[cfg(windows)]
			{
				std::fs::write(p, cwd.to_string_lossy().as_bytes()).ok();
			}
			#[cfg(unix)]
			{
				use std::os::unix::ffi::OsStrExt;
				std::fs::write(p, cwd.as_bytes()).ok();
			}
		}
		Term::goodbye(|| false).unwrap();
	}

	fn dispatch_key(&mut self, key: KeyEvent) {
		let key = Key::from(key);
		if Executor::new(&mut self.cx).handle(key) {
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

		COLLISION.store(false, Ordering::Relaxed);
		let frame = term.draw(|f| {
			yazi_plugin::scope(&self.cx, |_| {
				f.render_widget(Root::new(&self.cx), f.size());
			});

			if let Some((x, y)) = self.cx.cursor() {
				f.set_cursor(x, y);
			}
		})?;
		if !COLLISION.load(Ordering::Relaxed) {
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

	fn dispatch_stop(&mut self, state: bool, tx: Option<oneshot::Sender<()>>) {
		self.cx.manager.active_mut().preview.reset_image();
		if state {
			self.signals.stop_term(true);
			self.term = None;
		} else {
			self.term = Some(Term::start().unwrap());
			self.signals.stop_term(false);
			emit!(Render);
			Manager::_hover(None);
		}
		if let Some(tx) = tx {
			tx.send(()).ok();
		}
	}

	#[inline]
	fn dispatch_call(&mut self, exec: Vec<Exec>, layer: KeymapLayer) {
		if Executor::new(&mut self.cx).dispatch(&exec, layer) {
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

			Event::Select(opt, tx) => {
				self.cx.select.show(opt, tx);
				emit!(Render);
			}
			Event::Input(opt, tx) => {
				self.cx.input.show(opt, tx);
				emit!(Render);
			}

			Event::Open(targets, opener) => {
				if let Some(p) = &BOOT.chooser_file {
					let paths = targets.into_iter().fold(OsString::new(), |mut s, (p, _)| {
						s.push(p);
						s.push("\n");
						s
					});

					#[cfg(windows)]
					{
						std::fs::write(p, paths.to_string_lossy().as_bytes()).ok();
					}
					#[cfg(unix)]
					{
						use std::os::unix::ffi::OsStrExt;
						std::fs::write(p, paths.as_bytes()).ok();
					}
					return emit!(Quit(false));
				}

				if let Some(opener) = opener {
					tasks.file_open_with(&opener, &targets.into_iter().map(|(f, _)| f).collect::<Vec<_>>());
				} else {
					tasks.file_open(&targets);
				}
			}
			Event::Progress(progress) => {
				tasks.progress = progress;
				emit!(Render);
			}

			_ => unreachable!(),
		}
	}
}
