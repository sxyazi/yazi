use anyhow::{Ok, Result};
use crossterm::event::KeyEvent;
use tokio::sync::oneshot::{self};

use super::{root::Root, Ctx, Executor, Logs, Signals, Term};
use crate::{config::keymap::Key, core::{files::FilesOp, input::{Input, InputPos}, Event}, emit};

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
		self.cx.manager.current_mut().set_page(true);
		self.cx.manager.preview();
		emit!(Render);
	}

	async fn dispatch_module(&mut self, event: Event) {
		let manager = &mut self.cx.manager;
		match event {
			Event::Cd(path) => {
				manager.active_mut().cd(path).await;
			}
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
			Event::Pages(page) => {
				if manager.current().page == page {
					let targets = self.cx.manager.current().paginate().into_iter().map(|(_, f)| f).collect();
					self.cx.tasks.precache_mime(targets, &self.cx.manager.mimetype);
				}
			}
			Event::Mimetype(mimes) => {
				if manager.update_mimetype(mimes, &self.cx.tasks) {
					emit!(Render);
				}
			}
			Event::Hover => {
				if manager.preview() {
					emit!(Render);
				}
			}
			Event::Preview(file, data) => {
				manager.update_preview(file, data);
				emit!(Render);
			}

			Event::Input(mut opt, tx) => {
				opt.position = match opt.position {
					InputPos::Top => Input::top_position(),
					InputPos::Hovered => manager
						.hovered()
						.and_then(|h| manager.current().rect_current(&h.path))
						.map(|r| InputPos::Coords(r.x, r.y))
						.unwrap_or_else(|| Input::top_position()),
					p @ InputPos::Coords(..) => p,
				};

				self.cx.input.show(opt, tx);
				emit!(Render);
			}

			Event::Open(files) => {
				self.cx.tasks.file_open(files);
			}
			Event::Progress(percent, left) => {
				self.cx.tasks.progress = (percent, left);
				emit!(Render);
			}

			_ => unreachable!(),
		}
	}
}
