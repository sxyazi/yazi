use std::{collections::{BTreeSet, HashMap, HashSet}, path::PathBuf};

use tokio::fs;

use super::{PreviewData, Tab, Tabs, Watcher};
use crate::{core::{files::{File, FilesOp}, input::{Input, InputOpt}, manager::Folder, tasks::Precache}, emit};

pub struct Manager {
	tabs:   Tabs,
	yanked: (bool, HashSet<PathBuf>),

	watcher:  Watcher,
	mimetype: HashMap<PathBuf, String>,
}

impl Manager {
	pub fn new() -> Self {
		Self {
			tabs:   Tabs::new(),
			yanked: Default::default(),

			watcher:  Watcher::init(),
			mimetype: Default::default(),
		}
	}

	pub fn refresh(&mut self) {
		self.watcher.trigger(&self.current().cwd);
		if let Some(p) = self.parent() {
			self.watcher.trigger(&p.cwd);
		}
		emit!(Hover);

		let mut to_watch = BTreeSet::new();
		for tab in self.tabs.iter() {
			to_watch.insert(tab.current.cwd.clone());
			if let Some(ref p) = tab.parent {
				to_watch.insert(p.cwd.clone());
			}
			if let Some(ref h) = tab.current.hovered() {
				to_watch.insert(h.path.clone());
			}
		}
		self.watcher.watch(to_watch);
	}

	pub fn preview(&mut self) -> bool {
		let hovered = if let Some(h) = self.hovered() {
			h.clone()
		} else {
			return self.active_mut().preview.reset();
		};

		if hovered.meta.is_dir() {
			self.active_mut().preview.go(&hovered.path, "inode/directory");
			if self.active().history(&hovered.path).is_some() {
				emit!(Preview(hovered.path, PreviewData::Folder));
			}
		} else if let Some(mime) = self.mimetype.get(&hovered.path).cloned() {
			self.active_mut().preview.go(&hovered.path, &mime);
		} else {
			tokio::spawn(async move {
				if let Ok(mime) = Precache::mimetype(&vec![hovered.path.clone()]).await {
					if let Some(Some(mime)) = mime.first() {
						emit!(Mimetype(hovered.path, mime.clone()));
					}
				}
			});
		}
		false
	}

	pub fn close(&mut self) -> bool {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.idx());
		}

		emit!(Quit);
		return false;
	}

	pub fn yank(&mut self, cut: bool) -> bool {
		self.yanked.0 = cut;
		self.yanked.1.clear();
		self.yanked.1.extend(self.selected());
		false
	}

	#[inline]
	pub fn yanked(&self) -> &(bool, HashSet<PathBuf>) { &self.yanked }

	pub fn create(&self) -> bool {
		let pos = Input::top_position();
		let cwd = self.current().cwd.clone();

		tokio::spawn(async move {
			let result = emit!(Input(InputOpt {
				title:    "Create:".to_string(),
				value:    "".to_string(),
				position: pos,
			}))
			.await;

			if let Ok(name) = result {
				let path = cwd.join(&name);
				if name.ends_with('/') {
					fs::create_dir_all(path).await.ok();
				} else {
					fs::create_dir_all(path.parent().unwrap()).await.ok();
					fs::File::create(path).await.ok();
				}
			}
		});

		false
	}

	pub fn rename(&self) -> bool {
		let selected = self.selected();
		if selected.is_empty() {
			return false;
		}

		if selected.len() > 1 {
			return self.bulk_rename();
		}

		let rect = self.current().rect_current(&selected[0]).unwrap();
		tokio::spawn(async move {
			let result = emit!(Input(InputOpt {
				title:    "Rename:".to_string(),
				value:    selected[0].file_name().unwrap().to_string_lossy().to_string(),
				position: (rect.x, rect.y),
			}))
			.await;

			if let Ok(new) = result {
				let to = selected[0].parent().unwrap().join(new);
				fs::rename(&selected[0], to).await.ok();
			}
		});

		false
	}

	fn bulk_rename(&self) -> bool { false }

	pub fn selected(&self) -> Vec<PathBuf> {
		self
			.current()
			.selected()
			.or_else(|| self.hovered().map(|h| vec![h.path.clone()]))
			.unwrap_or_default()
	}

	pub async fn mimetype(&mut self, files: &Vec<PathBuf>) -> Vec<Option<String>> {
		let todo =
			files.iter().filter(|&p| !self.mimetype.contains_key(p)).cloned().collect::<Vec<_>>();
		if let Ok(mime) = Precache::mimetype(&todo).await {
			let mut it = todo.iter().zip(mime);
			while let Some((p, Some(m))) = it.next() {
				self.mimetype.insert(p.clone(), m);
			}
		}

		files.into_iter().map(|p| self.mimetype.get(p).cloned()).collect()
	}

	pub fn update_files(&mut self, op: FilesOp) -> bool {
		let path = op.path();
		let cwd = self.current().cwd.clone();

		let hovered = self.hovered().map(|h| h.path.clone());
		let folder = if cwd == path && !self.current().in_search {
			self.current_mut()
		} else if matches!(self.parent(), Some(p) if p.cwd == path) {
			self.active_mut().parent.as_mut().unwrap()
		} else {
			self.active_mut().history.entry(path.to_path_buf()).or_insert_with(|| Folder::new(&path))
		};

		let mut b = folder.update(op) || matches!(self.hovered(), Some(h) if h.path == path);
		b |= self.active_mut().parent.as_mut().map_or(false, |p| p.hover(&cwd));
		b |= hovered.as_ref().map_or(false, |h| self.current_mut().hover(h));

		if hovered != self.hovered().map(|h| h.path.clone()) {
			emit!(Hover);
		}
		b
	}

	pub fn update_mimetype(&mut self, path: PathBuf, mimetype: String) -> bool {
		if matches!(self.mimetype.get(&path), Some(m) if m == &mimetype) {
			return false;
		}

		self.mimetype.insert(path, mimetype);
		self.preview();
		true
	}

	pub fn update_preview(&mut self, path: PathBuf, data: PreviewData) -> bool {
		let hovered = if let Some(h) = self.current().hovered() {
			h.path.clone()
		} else {
			return self.active_mut().preview.reset();
		};

		if hovered != path {
			return false;
		}

		let preview = &mut self.active_mut().preview;
		preview.path = path;
		preview.data = data;
		true
	}
}

impl Manager {
	#[inline]
	pub fn tabs(&self) -> &Tabs { &self.tabs }

	#[inline]
	pub fn tabs_mut(&mut self) -> &mut Tabs { &mut self.tabs }

	#[inline]
	pub fn active(&self) -> &Tab { self.tabs.active() }

	#[inline]
	pub fn active_mut(&mut self) -> &mut Tab { self.tabs.active_mut() }

	#[inline]
	pub fn current(&self) -> &Folder { &self.tabs.active().current }

	#[inline]
	pub fn current_mut(&mut self) -> &mut Folder { &mut self.tabs.active_mut().current }

	#[inline]
	pub fn parent(&self) -> &Option<Folder> { &self.tabs.active().parent }

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.tabs.active().current.hovered() }
}
