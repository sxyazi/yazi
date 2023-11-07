use std::ffi::OsString;

use anyhow::{Ok, Result};
use crossterm::event::KeyEvent;
use tokio::sync::oneshot;
use yazi_config::{keymap::{Exec, Key, KeymapLayer}, BOOT};
use yazi_core::{emit, files::FilesOp, input::InputMode, Ctx, Event};
use yazi_shared::{expand_url, Term};

use crate::{Executor, Logs, Root, Signals};

pub(super) struct App {
	cx:      Ctx,
	term:    Option<Term>,
	signals: Signals,
}

impl App {
	pub(super) async fn run() -> Result<()> {
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
				Event::Render(_) => app.dispatch_render(),
				Event::Resize(..) => app.dispatch_resize(),
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

	fn dispatch_render(&mut self) {
		if let Some(term) = &mut self.term {
			_ = term.draw(|f| {
				yazi_plugin::scope(&self.cx, |_| {
					f.render_widget(Root::new(&self.cx), f.size());
				});

				if let Some((x, y)) = self.cx.cursor() {
					f.set_cursor(x, y);
				}
			});
		}
	}

	fn dispatch_resize(&mut self) {
		self.cx.manager.current_mut().set_page(true);
		self.cx.manager.active_mut().preview.reset(|_| true);
		self.cx.manager.peek(true, self.cx.image_layer());
		emit!(Render);
	}

	fn dispatch_stop(&mut self, state: bool, tx: Option<oneshot::Sender<()>>) {
		self.cx.manager.active_mut().preview.reset(|l| l.is_image());
		if state {
			self.signals.stop_term(true);
			self.term = None;
		} else {
			self.term = Some(Term::start().unwrap());
			self.signals.stop_term(false);
			emit!(Render);
			emit!(Hover);
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
			Event::Cd(url) => {
				manager.active_mut().cd(expand_url(url));
			}
			Event::Refresh => {
				manager.refresh();
			}
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
				if manager.current().page == page {
					let targets = self.cx.manager.current().paginate();
					tasks.precache_mime(targets, &self.cx.manager.mimetype);
				}
			}
			Event::Mimetype(mimes) => {
				if manager.update_mimetype(mimes, tasks) {
					emit!(Render);
					emit!(Peek);
				}
			}
			Event::Hover(url) => {
				if manager.current_mut().repos(url) {
					emit!(Render);
				}
				emit!(Peek);
			}
			Event::Peek(sequent) => {
				if let Some((max, url)) = sequent {
					manager.active_mut().update_peek(max, url);
					self.cx.manager.peek(true, self.cx.image_layer());
				} else {
					self.cx.manager.peek(false, self.cx.image_layer());
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
