use std::{borrow::Cow, collections::{BTreeMap, BTreeSet}, ffi::{OsStr, OsString}, mem, path::{Path, PathBuf}};

use anyhow::{Error, Result};
use config::{open::Opener, MANAGER};
use futures::StreamExt;
use shared::{Defer, MIME_DIR};
use tokio::task::JoinHandle;

use super::{Folder, Mode, Preview, PreviewLock};
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
			parent:  path.parent().map(Folder::new),

			search: None,

			history: Default::default(),
			preview: Default::default(),
		}
	}

	pub fn escape(&mut self) -> bool {
		if let Some((_, indices)) = self.mode.visual() {
			self.current.files.select_index(indices, Some(self.mode.is_select()));
			self.mode = Mode::Normal;
			return true;
		}

		if self.select_all(Some(false)) {
			return true;
		}

		self.search_stop()
	}

	pub fn arrow(&mut self, step: isize) -> bool {
		let ok = if step > 0 {
			self.current.next(step as usize)
		} else {
			self.current.prev(step.unsigned_abs())
		};
		if !ok {
			return false;
		}

		// Visual selection
		if let Some((start, items)) = self.mode.visual_mut() {
			let after = self.current.cursor();

			items.clear();
			for i in start.min(after)..=after.max(start) {
				items.insert(i);
			}
		}

		emit!(Hover);
		true
	}

	pub async fn cd(&mut self, mut target: PathBuf) -> bool {
		let Ok(file) = File::from(&target).await else {
			return false;
		};

		let mut hovered = None;
		if !file.is_dir() {
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

	pub fn cd_interactive(&mut self, target: PathBuf) -> bool {
		tokio::spawn(async move {
			let result =
				emit!(Input(InputOpt::top("Change directory:").with_value(target.to_string_lossy())));

			if let Ok(target) = result.await {
				emit!(Cd(PathBuf::from(target)));
			}
		});
		false
	}

	pub fn enter(&mut self) -> bool {
		let Some(hovered) = self.current.hovered.clone() else {
			return false;
		};
		if !hovered.is_dir() {
			return false;
		}

		let rep = self.history_new(hovered.path());
		let rep = mem::replace(&mut self.current, rep);
		if !rep.in_search {
			self.history.insert(rep.cwd.clone(), rep);
		}

		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}
		self.parent = Some(self.history_new(hovered.path().parent().unwrap()));

		emit!(Refresh);
		true
	}

	pub fn leave(&mut self) -> bool {
		let current = self
			.current
			.hovered
			.as_ref()
			.and_then(|h| h.path().parent())
			.and_then(|p| if p == self.current.cwd { None } else { Some(p) })
			.or_else(|| self.current.cwd.parent());

		let Some(current) = current.map(Path::to_path_buf) else {
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

	pub fn select(&mut self, state: Option<bool>) -> bool {
		if let Some(ref hovered) = self.current.hovered {
			return self.current.files.select(hovered.path(), state);
		}
		false
	}

	pub fn select_all(&mut self, state: Option<bool>) -> bool {
		self.current.files.select_many(None, state)
	}

	pub fn visual_mode(&mut self, unset: bool) -> bool {
		let idx = self.current.cursor();

		if unset {
			self.mode = Mode::Unset(idx, BTreeSet::from([idx]));
		} else {
			self.mode = Mode::Select(idx, BTreeSet::from([idx]));
		};
		true
	}

	pub fn copy(&self, type_: &str) -> bool {
		let mut s = OsString::new();
		let mut it = self.selected().into_iter().peekable();
		while let Some(f) = it.next() {
			s.push(match type_ {
				"path" => f.path().as_os_str(),
				"dirname" => f.path().parent().map_or(OsStr::new(""), |p| p.as_os_str()),
				"filename" => f.path().file_name().unwrap_or(OsStr::new("")),
				"name_without_ext" => f.path().file_stem().unwrap_or(OsStr::new("")),
				_ => return false,
			});
			if it.peek().is_some() {
				s.push("\n");
			}
		}

		futures::executor::block_on(external::clipboard_set(s)).ok();
		false
	}

	pub fn search(&mut self, grep: bool) -> bool {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}

		let cwd = self.current.cwd.clone();
		let hidden = self.current.files.show_hidden();

		self.search = Some(tokio::spawn(async move {
			let subject = emit!(Input(InputOpt::top("Search:"))).await?;

			let mut rx = if grep {
				external::rg(external::RgOpt { cwd: cwd.clone(), hidden, subject })
			} else {
				external::fd(external::FdOpt { cwd: cwd.clone(), hidden, glob: false, subject })
			}?;

			emit!(Files(FilesOp::search_empty(&cwd)));
			while let Some(chunk) = rx.next().await {
				emit!(Files(FilesOp::Search(cwd.clone(), Files::read(&chunk).await)));
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

	pub fn shell(&self, exec: &str, block: bool, confirm: bool) -> bool {
		let selected: Vec<_> = self
			.selected()
			.into_iter()
			.map(|f| (f.path().as_os_str().to_owned(), Default::default()))
			.collect();

		let mut exec = exec.to_owned();
		tokio::spawn(async move {
			if !confirm || exec.is_empty() {
				let result = emit!(Input(InputOpt::top("Shell:").with_value(&exec).with_highlight()));
				match result.await {
					Ok(e) => exec = e,
					Err(_) => return,
				}
			}

			emit!(Open(
				selected,
				Some(Opener { exec, block, display_name: Default::default(), spread: true })
			));
		});

		false
	}

	pub fn update_peek(&mut self, step: isize, path: Option<PathBuf>) {
		let Some(ref hovered) = self.current.hovered else {
			return;
		};

		if path.as_ref().map(|p| p != hovered.path()).unwrap_or(false) {
			return;
		} else if !self.preview.arrow(step, path.is_some()) {
			return;
		} else if !matches!(&self.preview.lock, Some(l) if l.mime == MIME_DIR) {
			return;
		}

		let path = &self.preview.lock.as_ref().unwrap().path;
		if let Some(folder) = self.history(path) {
			let max = folder.files.len().saturating_sub(MANAGER.layout.preview_height());
			if self.preview.skip() > max {
				self.preview.arrow(max as isize, true);
			}
		}
	}

	pub fn update_preview(&mut self, lock: PreviewLock) -> bool {
		let Some(hovered) = self.current.hovered.as_ref().map(|h| h.path()) else {
			return self.preview.reset();
		};

		if lock.path != *hovered {
			return false;
		}

		self.preview.lock = Some(lock);
		true
	}
}

impl Tab {
	#[inline]
	pub fn mode(&self) -> &Mode { &self.mode }

	#[inline]
	pub fn name(&self) -> &str {
		self
			.current
			.cwd
			.file_name()
			.and_then(|n| n.to_str())
			.or_else(|| self.current.cwd.to_str())
			.unwrap_or_default()
	}

	pub fn selected(&self) -> Vec<&File> {
		let mode = self.mode();
		let pending = mode.visual().map(|(_, p)| Cow::Borrowed(p)).unwrap_or_default();

		let selected = self.current.files.selected(&pending, mode.is_unset());
		if selected.is_empty() {
			self.current.hovered.as_ref().map(|h| vec![h]).unwrap_or_default()
		} else {
			selected
		}
	}

	#[inline]
	pub fn in_selecting(&self) -> bool {
		self.mode().is_visual() || self.current.files.has_selected()
	}

	#[inline]
	pub fn history(&self, path: &Path) -> Option<&Folder> { self.history.get(path) }

	#[inline]
	pub fn history_new(&mut self, path: &Path) -> Folder {
		self.history.remove(path).unwrap_or_else(|| Folder::new(path))
	}

	#[inline]
	pub fn preview(&self) -> &Preview { &self.preview }

	#[inline]
	pub fn preview_reset(&mut self) -> bool { self.preview.reset() }

	#[inline]
	pub fn preview_reset_image(&mut self) -> bool { self.preview.reset_image() }
}
