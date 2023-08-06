use std::{collections::BTreeMap, mem, path::{Path, PathBuf}};

use anyhow::{Error, Result};
use shared::Defer;
use tokio::task::JoinHandle;

use super::{Folder, Mode, Preview};
use crate::{emit, external::{self, FzfOpt, ZoxideOpt}, files::{File, Files, FilesOp}, input::InputOpt, Event, BLOCKER};

pub struct Tab {
	pub(super) mode:    Mode,
	pub(super) current: Folder,
	pub(super) parent:  Option<Folder>,

	search: Option<JoinHandle<Result<()>>>,

	pub(super) history: BTreeMap<PathBuf, Folder>,
	pub(super) preview: Preview,
}

impl Tab {
	pub fn new(path: &Path) -> Self {
		Self {
			mode:    Default::default(),
			current: Folder::new(path),
			parent:  path.parent().map(|p| Folder::new(p)),

			search: None,

			history: Default::default(),
			preview: Default::default(),
		}
	}

	pub fn escape(&mut self) -> bool {
		if matches!(self.mode, Mode::Select(_) | Mode::Unset(_)) {
			self.mode = Mode::Normal;
			return true;
		}

		if self.select_all(Some(false)) {
			return true;
		}

		self.search_stop()
	}

	pub fn arrow(&mut self, step: isize) -> bool {
		let before = self.current.cursor();
		let ok = if step > 0 {
			self.current.next(step as usize)
		} else {
			self.current.prev(step.abs() as usize)
		};
		if !ok {
			return false;
		}

		// Visual selection
		if let Some(start) = self.mode.start() {
			let after = self.current.cursor();
			if (after > before && before < start) || (after < before && before > start) {
				for i in before.min(start)..=start.max(before) {
					self.current.select(Some(i), Some(false));
				}
			}
			for i in start.min(after)..=after.max(start) {
				self.current.select(Some(i), Some(true));
			}
		}

		emit!(Hover);
		true
	}

	pub async fn cd(&mut self, mut target: PathBuf) -> bool {
		let file = if let Ok(f) = File::from(&target).await {
			f
		} else {
			return false;
		};

		let mut hovered = None;
		if !file.meta.is_dir() {
			hovered = Some(file);
			target = target.parent().unwrap().to_path_buf();
		}

		if self.current.cwd == target {
			if hovered.map(|h| self.current.hover_force(h)).unwrap_or(false) {
				emit!(Hover);
			}
			return false;
		}

		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		let rep = self.history_new(&target);
		let rep = mem::replace(&mut self.current, rep);
		if !rep.in_search {
			self.history.insert(rep.cwd.clone(), rep);
		}

		if let Some(parent) = target.parent() {
			self.parent = Some(self.history_new(parent));
		}

		if let Some(h) = hovered {
			self.current.hover_force(h);
		}
		emit!(Refresh);
		true
	}

	pub fn enter(&mut self) -> bool {
		let hovered = if let Some(ref h) = self.current.hovered {
			h.clone()
		} else {
			return false;
		};
		if !hovered.meta.is_dir() {
			return false;
		}

		let rep = self.history_new(&hovered.path);
		let rep = mem::replace(&mut self.current, rep);
		if !rep.in_search {
			self.history.insert(rep.cwd.clone(), rep);
		}

		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}
		self.parent = Some(self.history_new(hovered.path.parent().unwrap()));

		emit!(Refresh);
		true
	}

	pub fn leave(&mut self) -> bool {
		let current = self
			.current
			.hovered
			.as_ref()
			.and_then(|h| h.path.parent())
			.and_then(|p| if p == self.current.cwd { None } else { Some(p) })
			.or_else(|| self.current.cwd.parent());

		let current = if let Some(c) = current {
			c.to_owned()
		} else {
			return false;
		};

		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}
		if let Some(parent) = current.parent() {
			self.parent = Some(self.history_new(parent));
		}

		let rep = self.history_new(&current);
		let rep = mem::replace(&mut self.current, rep);
		if !rep.in_search {
			self.history.insert(rep.cwd.clone(), rep);
		}

		emit!(Refresh);
		true
	}

	// TODO
	pub fn back(&mut self) -> bool { false }

	// TODO
	pub fn forward(&mut self) -> bool { false }

	pub fn search(&mut self, grep: bool) -> bool {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}

		let cwd = self.current.cwd.clone();
		let hidden = self.current.files.show_hidden;

		self.search = Some(tokio::spawn(async move {
			let subject = emit!(Input(InputOpt::top("Search:"))).await?;

			let mut rx = if grep {
				external::rg(external::RgOpt { cwd: cwd.clone(), hidden, subject })
			} else {
				external::fd(external::FdOpt { cwd: cwd.clone(), hidden, glob: false, subject })
			}?;

			emit!(Files(FilesOp::search_empty(&cwd)));
			while let Some(chunk) = rx.recv().await {
				if chunk.is_empty() {
					break;
				}
				emit!(Files(FilesOp::Search(cwd.clone(), Files::read(chunk).await)));
			}
			Ok(())
		}));
		true
	}

	pub fn search_stop(&mut self) -> bool {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}
		if self.current.in_search {
			self.preview_reset_image();

			let cwd = self.current.cwd.clone();
			let rep = self.history_new(&cwd);
			drop(mem::replace(&mut self.current, rep));
			emit!(Refresh);
		}
		false
	}

	pub fn jump(&self, global: bool) -> bool {
		let cwd = self.current.cwd.clone();

		tokio::spawn(async move {
			let _guard = BLOCKER.acquire().await.unwrap();
			let _defer = Defer::new(|| Event::Stop(false, None).emit());
			emit!(Stop(true)).await;

			let rx =
				if global { external::fzf(FzfOpt { cwd }) } else { external::zoxide(ZoxideOpt { cwd }) }?;

			if let Ok(target) = rx.await? {
				emit!(Cd(target));
			}
			Ok::<(), Error>(())
		});
		false
	}

	pub fn select(&mut self, state: Option<bool>) -> bool {
		let idx = Some(self.current.cursor());
		self.current.select(idx, state)
	}

	pub fn select_all(&mut self, state: Option<bool>) -> bool { self.current.select(None, state) }

	pub fn visual_mode(&mut self, unset: bool) -> bool {
		let idx = self.current.cursor();

		if unset {
			self.mode = Mode::Unset(idx);
			self.current.select(Some(idx), Some(false));
		} else {
			self.mode = Mode::Select(idx);
			self.current.select(Some(idx), Some(true));
		};
		true
	}
}

impl Tab {
	#[inline]
	pub fn mode(&self) -> &Mode { &self.mode }

	#[inline]
	pub fn history(&self, path: &Path) -> Option<&Folder> { self.history.get(path) }

	#[inline]
	pub fn history_new(&mut self, path: &Path) -> Folder {
		self.history.remove(path).unwrap_or_else(|| Folder::new(path))
	}

	#[inline]
	pub fn preview(&self) -> &Preview { &self.preview }

	#[inline]
	pub fn preview_reset_image(&mut self) -> bool { self.preview.reset_image() }
}
