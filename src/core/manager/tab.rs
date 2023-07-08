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

		let current = self.history.remove(&hovered.path).unwrap_or_else(|| Folder::new(&hovered.path));
		let parent = mem::replace(&mut self.current, current);

		if self.parent.is_none() {
			self.parent = Some(parent);
		} else {
			let cwd = self.parent.as_ref().unwrap().cwd.clone();
			let pparent = mem::replace(self.parent.as_mut().unwrap(), parent);
			self.history.insert(cwd, pparent);
		}

		emit!(Refresh);
		true
	}

	pub fn leave(&mut self) -> bool {
		let parent = if let Some(p) = &self.parent {
			p.cwd.clone()
		} else {
			return false;
		};

		let pparent = parent.parent().map(|p| self.history.remove(p).unwrap_or_else(|| Folder::new(p)));

		let cwd = self.current.cwd.clone();
		let parent = mem::replace(&mut self.parent, pparent).unwrap();
		let current = mem::replace(&mut self.current, parent);
		self.history.insert(cwd, current);

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
	pub fn preview(&self) -> &Preview { &self.preview }
}
