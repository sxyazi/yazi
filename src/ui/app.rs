use anyhow::{Ok, Result};
use crossterm::event::KeyEvent;
use tokio::sync::oneshot::{self};

use super::{root::Root, Ctx, Executor, Logs, Signals, Term};
use crate::{config::keymap::Key, core::{files::FilesOp, Event}, emit};

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
				Event::Stop(state, tx) => app.dispatch_stop(state, tx),
				Event::Key(key) => app.dispatch_key(key),
				Event::Render(_) => app.dispatch_render(),
				Event::Resize(..) => app.dispatch_resize(),
				event => app.dispatch_module(event).await,
			}
		}
		Ok(())
	}

	fn dispatch_stop(&mut self, state: bool, tx: oneshot::Sender<()>) {
		if state {
			self.signals.stop_term(true);
			self.term = None;
		} else {
			self.term = Some(Term::start().unwrap());
			self.signals.stop_term(false);
			emit!(Render);
		}
		tx.send(()).ok();
	}

	fn dispatch_key(&mut self, key: KeyEvent) {
		let key = Key::from(key);
		if Executor::handle(&mut self.cx, key) {
			emit!(Render);
		}
	}

	fn dispatch_render(&mut self) {
		if let Some(term) = &mut self.term {
			let _ = term.draw(|f| {
				f.render_widget(Root::new(&mut self.cx), f.size());

				if let Some((x, y)) = self.cx.cursor {
					f.set_cursor(x, y);
				}
			});
		}
	}

	fn dispatch_resize(&mut self) {
		self.cx.manager.preview();
		emit!(Render);
	}

	async fn dispatch_module(&mut self, event: Event) {
		let manager = &mut self.cx.manager;
		match event {
			Event::Refresh => {
				manager.refresh();
			}
			Event::Files(op) => {
				let b = match op {
					FilesOp::Read(..) => manager.update_read(op),
					FilesOp::IOErr(..) => manager.update_ioerr(op),
					FilesOp::Search(..) => manager.update_search(op),
				};
				if b {
					emit!(Render);
				}
			}
			Event::Hover => {
				if manager.preview() {
					emit!(Render);
				}
			}
			Event::Mimetype(file, mime) => {
				if manager.update_mimetype(file, mime) {
					emit!(Render);
				}
			}
			Event::Preview(file, data) => {
				manager.update_preview(file, data);
				emit!(Render);
			}

			Event::Input(opt, tx) => {
				self.cx.input.show(opt, tx);
				emit!(Render);
			}

			Event::Open(files) => {
				let mime = self.cx.manager.mimetype(&files).await;
				let targets = files.into_iter().zip(mime).map_while(|(f, m)| m.map(|m| (f, m))).collect();
				self.cx.tasks.file_open(targets);
			}
			Event::Progress(percent, left) => {
				self.cx.tasks.update_progress(percent, left);
				emit!(Render);
			}

			_ => unreachable!(),
		}
	}
}
