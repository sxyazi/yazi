use core::{emit, files::FilesOp, input::InputMode, Event};
use std::ffi::OsString;

use anyhow::{Ok, Result};
use config::{keymap::{Control, Key, KeymapLayer}, BOOT};
use crossterm::event::KeyEvent;
use shared::{absolute_path, Term};
use tokio::sync::oneshot;

use crate::{Ctx, Executor, Logs, Root, Signals};

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
		let mut app = Self { cx: Ctx::new(), term: Some(term), signals };

		while let Some(event) = app.signals.recv().await {
			match event {
				Event::Quit => {
					app.dispatch_quit();
					break;
				}
				Event::Key(key) => app.dispatch_key(key),
				Event::Paste(str) => app.dispatch_paste(str),
				Event::Render(_) => app.dispatch_render(),
				Event::Resize(..) => app.dispatch_resize(),
				Event::Stop(state, tx) => app.dispatch_stop(state, tx),
				Event::Ctrl(ctrl, layer) => app.dispatch_ctrl(ctrl, layer),
				event => app.dispatch_module(event),
			}
		}
		Ok(())
	}

	fn dispatch_quit(&mut self) {
		if let Some(p) = &BOOT.cwd_file {
			let cwd = self.cx.manager.cwd().as_os_str();

			#[cfg(target_os = "windows")]
			{
				std::fs::write(p, cwd.to_string_lossy().as_bytes()).ok();
			}
			#[cfg(not(target_os = "windows"))]
			{
				use std::os::unix::ffi::OsStrExt;
				std::fs::write(p, cwd.as_bytes()).ok();
			}
		}
	}

	fn dispatch_key(&mut self, key: KeyEvent) {
		let key = Key::from(key);
		if Executor::handle(&mut self.cx, key) {
			emit!(Render);
		}
	}

	fn dispatch_paste(&mut self, str: String) {
		if self.cx.layer() == KeymapLayer::Input {
			let input = &mut self.cx.input;
			if input.mode() == InputMode::Insert && input.type_str(&str) {
				emit!(Render);
			}
		}
	}

	fn dispatch_render(&mut self) {
		if let Some(term) = &mut self.term {
			let _ = term.draw(|f| {
				f.render_widget(Root::new(&self.cx), f.size());

				if let Some((x, y)) = self.cx.cursor() {
					f.set_cursor(x, y);
				}
			});
		}
	}

	fn dispatch_resize(&mut self) {
		self.cx.manager.current_mut().set_page(true);
		self.cx.manager.active_mut().preview_reset();
		self.cx.manager.peek(true, self.cx.image_layer());
		emit!(Render);
	}

	fn dispatch_stop(&mut self, state: bool, tx: Option<oneshot::Sender<()>>) {
		self.cx.manager.active_mut().preview_reset_image();
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
	fn dispatch_ctrl(&mut self, ctrl: Control, layer: KeymapLayer) {
		if Executor::dispatch(&mut self.cx, &ctrl.exec, layer) {
			emit!(Render);
		}
	}

	fn dispatch_module(&mut self, event: Event) {
		let manager = &mut self.cx.manager;
		let tasks = &mut self.cx.tasks;
		match event {
			Event::Cd(path) => {
				futures::executor::block_on(async {
					manager.active_mut().cd(absolute_path(path)).await;
				});
			}
			Event::Refresh => {
				manager.refresh();
			}
			Event::Files(op) => {
				let read = matches!(op, FilesOp::Read(..));
				let b = match op {
					FilesOp::Read(..) => manager.update_read(op),
					FilesOp::Size(..) => manager.update_read(op),
					FilesOp::IOErr(..) => manager.update_ioerr(op),
				};
				if b {
					emit!(Render);
				}
				if read {
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
			Event::Hover(file) => {
				if manager.update_hover(file) {
					emit!(Render);
				}
				emit!(Peek);
			}
			Event::Peek(skip, sequent) => {
				let b = sequent.is_some();
				manager.active_mut().update_peek(skip as isize, sequent);
				self.cx.manager.peek(b, self.cx.image_layer());
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

					#[cfg(target_os = "windows")]
					{
						std::fs::write(p, paths.to_string_lossy().as_bytes()).ok();
					}
					#[cfg(not(target_os = "windows"))]
					{
						use std::os::unix::ffi::OsStrExt;
						std::fs::write(p, paths.as_bytes()).ok();
					}
					return emit!(Quit);
				}

				if let Some(opener) = opener {
					tasks.file_open_with(&opener, &targets.into_iter().map(|(f, _)| f).collect::<Vec<_>>());
				} else {
					tasks.file_open(&targets);
				}
			}
			Event::Progress(percent, left) => {
				tasks.progress = (percent, left);
				emit!(Render);
			}

			_ => unreachable!(),
		}
	}
}
