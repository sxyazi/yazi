use anyhow::{Ok, Result};
use crossterm::event::KeyEvent;
use tokio::sync::oneshot::{self};

use super::{root::Root, Ctx, Executor, Logs, Signals, Term};
use crate::{config::keymap::{Control, Key, KeymapLayer}, core::{files::FilesOp, input::InputMode, Event}, emit, misc::absolute_path};

pub struct App {
	cx:      Ctx,
	term:    Option<Term>,
	signals: Signals,
}

impl App {
	pub async fn run() -> Result<()> {
		let _log = Logs::init()?;
		let term = Term::start()?;

		let signals = Signals::start()?;
		let mut app = Self { cx: Ctx::new(), term: Some(term), signals };

		while let Some(event) = app.signals.rx.recv().await {
			match event {
				Event::Quit => break,
				Event::Key(key) => app.dispatch_key(key),
				Event::Paste(str) => app.dispatch_paste(str),
				Event::Render(_) => app.dispatch_render(),
				Event::Resize(..) => app.dispatch_resize(),
				Event::Stop(state, tx) => app.dispatch_stop(state, tx),
				Event::Ctrl(ctrl, layer) => app.dispatch_ctrl(ctrl, layer),
				event => app.dispatch_module(event).await,
			}
		}
		Ok(())
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
				f.render_widget(Root::new(&mut self.cx), f.size());

				if let Some((x, y)) = self.cx.cursor() {
					f.set_cursor(x, y);
				}
			});
		}
	}

	fn dispatch_resize(&mut self) {
		self.cx.manager.current_mut().set_page(true);
		self.cx.manager.preview(self.cx.image_layer());
		emit!(Render);
	}

	fn dispatch_stop(&mut self, state: bool, tx: Option<oneshot::Sender<()>>) {
		if state {
			self.signals.stop_term(true);
			self.term = None;
		} else {
			self.term = Some(Term::start().unwrap());
			self.signals.stop_term(false);
			emit!(Render);
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

	async fn dispatch_module(&mut self, event: Event) {
		let manager = &mut self.cx.manager;
		let tasks = &mut self.cx.tasks;
		match event {
			Event::Cd(path) => {
				manager.active_mut().cd(absolute_path(path).await).await;
			}
			Event::Refresh => {
				manager.refresh();
			}
			Event::Files(op) => {
				let calc = matches!(op, FilesOp::Read(..) | FilesOp::Search(..));
				let b = match op {
					FilesOp::Read(..) => manager.update_read(op),
					FilesOp::Sort(..) => manager.update_read(op),
					FilesOp::Search(..) => manager.update_search(op),
					FilesOp::IOErr(..) => manager.update_ioerr(op),
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
					let targets = self.cx.manager.current().paginate().into_iter().map(|(_, f)| f).collect();
					tasks.precache_mime(targets, &self.cx.manager.mimetype);
				}
			}
			Event::Mimetype(mimes) => {
				if manager.update_mimetype(mimes, tasks) {
					emit!(Render);
					self.cx.manager.preview(self.cx.image_layer());
				}
			}
			Event::Hover(file) => {
				if file.map(|f| manager.current_mut().hover_force(f)).unwrap_or(false) {
					emit!(Render);
				}
				self.cx.manager.preview(self.cx.image_layer());
			}
			Event::Preview(path, mime, data) => {
				manager.update_preview(path, mime, data);
				emit!(Render);
			}

			Event::Select(mut opt, tx) => {
				opt.position = self.cx.position(opt.position);
				self.cx.select.show(opt, tx);
				emit!(Render);
			}
			Event::Input(mut opt, tx) => {
				opt.position = self.cx.position(opt.position);
				self.cx.input.show(opt, tx);
				emit!(Render);
			}

			Event::Open(targets, opener) => {
				if let Some(opener) = opener {
					tasks.file_open_with(&opener, &targets.iter().map(|(f, _)| f).collect::<Vec<_>>());
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
