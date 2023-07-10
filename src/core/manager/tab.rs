use std::{collections::BTreeMap, mem, path::{Path, PathBuf}};

use super::{Folder, Mode, Preview};
use crate::emit;

pub struct Tab {
	pub(super) current: Folder,
	pub(super) parent:  Option<Folder>,

	pub(super) mode: Mode,

	pub(super) history: BTreeMap<PathBuf, Folder>,
	pub(super) preview: Preview,
}

impl Tab {
	pub fn new(path: &Path) -> Self {
		Self {
			current: Folder::new(path),
			parent:  path.parent().map(|p| Folder::new(p)),

			mode: Default::default(),

			history: Default::default(),
			preview: Preview::new(),
		}
	}

	pub fn escape(&mut self) -> bool {
		if matches!(self.mode, Mode::Select(_) | Mode::Unselect(_)) {
			self.mode = Mode::Normal;
			return true;
		}

		self.select_all(Some(false))
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

	pub fn enter(&mut self) -> bool {
		let hovered = if let Some(h) = self.current.hovered() {
			h.clone()
		} else {
			return false;
		};
		if !hovered.meta.is_dir() {
			emit!(Open(self.current.selected().unwrap_or(vec![hovered.path])));
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
			.hovered()
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

	pub fn back(&mut self) -> bool { todo!() }

	pub fn forward(&mut self) -> bool { todo!() }

	pub fn select(&mut self, state: Option<bool>) -> bool {
		let idx = Some(self.current.cursor());
		self.current.select(idx, state)
	}

	pub fn select_all(&mut self, state: Option<bool>) -> bool { self.current.select(None, state) }

	pub fn visual_mode(&mut self, unsel: bool) -> bool {
		let idx = self.current.cursor();

		if unsel {
			self.mode = Mode::Unselect(idx);
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
}
