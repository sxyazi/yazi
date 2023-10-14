use std::{borrow::Cow, collections::{BTreeMap, BTreeSet}, ffi::{OsStr, OsString}, mem, time::Duration};

use anyhow::{bail, Error, Result};
use config::{keymap::{Exec, KeymapLayer}, open::Opener, MANAGER};
use shared::{Debounce, Defer, InputError, Url};
use tokio::{pin, task::JoinHandle};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use super::{Backstack, Finder, FinderCase, Folder, Mode, Preview, PreviewLock};
use crate::{emit, external::{self, FzfOpt, ZoxideOpt}, files::{File, FilesOp, FilesSorter}, input::InputOpt, Event, Step, BLOCKER};

pub struct Tab {
	pub mode:    Mode,
	pub current: Folder,
	pub parent:  Option<Folder>,

	pub(super) backstack: Backstack<Url>,
	pub(super) history:   BTreeMap<Url, Folder>,
	pub(super) preview:   Preview,

	finder:                 Option<Finder>,
	search:                 Option<JoinHandle<Result<()>>>,
	pub(super) sorter:      FilesSorter,
	pub(super) show_hidden: bool,
}

impl From<Url> for Tab {
	fn from(url: Url) -> Self {
		let parent = url.parent_url().map(Folder::from);

		Self {
			mode: Default::default(),
			current: Folder::from(url.clone()),
			parent,

			backstack: Backstack::new(url),
			history: Default::default(),
			preview: Default::default(),

			finder: None,
			search: None,
			sorter: Default::default(),
			show_hidden: MANAGER.show_hidden,
		}
	}
}

impl From<&Url> for Tab {
	fn from(url: &Url) -> Self { Self::from(url.clone()) }
}

impl Tab {
	pub fn escape(&mut self) -> bool {
		if self.finder.is_some() {
			self.finder = None;
			return true;
		}

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

	pub fn arrow(&mut self, step: Step) -> bool {
		let ok = if step.is_positive() { self.current.next(step) } else { self.current.prev(step) };
		if !ok {
			return false;
		}

		// Visual selection
		if let Some((start, items)) = self.mode.visual_mut() {
			let after = self.current.cursor;

			items.clear();
			for i in start.min(after)..=after.max(start) {
				items.insert(i);
			}
		}

		emit!(Hover);
		true
	}

	// TODO: change to sync, and remove `Event::Cd`
	pub async fn cd(&mut self, mut target: Url) -> bool {
		let Ok(file) = File::from(target.clone()).await else {
			return false;
		};

		let mut hovered = None;
		if !file.is_dir() {
			hovered = Some(file.url_owned());
			target = target.parent_url().unwrap();
			emit!(Files(FilesOp::Creating(target.clone(), file.into_map())));
		}

		// Already in target
		if self.current.cwd == target {
			if let Some(h) = hovered {
				emit!(Hover(h));
			}
			return false;
		}

		// Take parent to history
		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Current
		let rep = self.history_new(&target);
		let rep = mem::replace(&mut self.current, rep);
		if rep.cwd.is_regular() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Parent
		if let Some(parent) = target.parent_url() {
			self.parent = Some(self.history_new(&parent));
		}

		// Hover the file
		if let Some(h) = hovered {
			emit!(Hover(h));
		}

		// Backstack
		if target.is_regular() {
			self.backstack.push(target.clone());
		}

		emit!(Refresh);
		true
	}

	pub fn cd_interactive(&mut self, target: Url) -> bool {
		tokio::spawn(async move {
			let mut result =
				emit!(Input(InputOpt::top("Change directory:").with_value(target.to_string_lossy())));

			if let Some(Ok(s)) = result.recv().await {
				emit!(Cd(Url::from(s.trim())));
			}
		});
		false
	}

	pub fn enter(&mut self) -> bool {
		let Some(hovered) = self.current.hovered().filter(|h| h.is_dir()).map(|h| h.url_owned()) else {
			return false;
		};

		// Current
		let rep = self.history_new(&hovered);
		let rep = mem::replace(&mut self.current, rep);
		if rep.cwd.is_regular() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Parent
		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}
		self.parent = Some(self.history_new(&hovered.parent_url().unwrap()));

		// Backstack
		self.backstack.push(hovered);

		emit!(Refresh);
		true
	}

	pub fn leave(&mut self) -> bool {
		let current = self
			.current
			.hovered()
			.and_then(|h| h.parent())
			.filter(|p| *p != self.current.cwd)
			.or_else(|| self.current.cwd.parent_url());

		let Some(current) = current else {
			return false;
		};

		// Parent
		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}
		if let Some(parent) = current.parent_url() {
			self.parent = Some(self.history_new(&parent));
		}

		// Current
		let rep = self.history_new(&current);
		let rep = mem::replace(&mut self.current, rep);
		if rep.cwd.is_regular() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Backstack
		self.backstack.push(current);

		emit!(Refresh);
		true
	}

	pub fn back(&mut self) -> bool {
		if let Some(url) = self.backstack.shift_backward().cloned() {
			futures::executor::block_on(self.cd(url));
		}
		false
	}

	pub fn forward(&mut self) -> bool {
		if let Some(url) = self.backstack.shift_forward().cloned() {
			futures::executor::block_on(self.cd(url));
		}
		false
	}

	pub fn select(&mut self, state: Option<bool>) -> bool {
		if let Some(u) = self.current.hovered().map(|h| h.url_owned()) {
			return self.current.files.select(&u, state);
		}
		false
	}

	pub fn select_all(&mut self, state: Option<bool>) -> bool { self.current.files.select_all(state) }

	pub fn visual_mode(&mut self, unset: bool) -> bool {
		let idx = self.current.cursor;

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
				"path" => f.url().as_os_str(),
				"dirname" => f.url().parent().map_or(OsStr::new(""), |p| p.as_os_str()),
				"filename" => f.name().unwrap_or(OsStr::new("")),
				"name_without_ext" => f.stem().unwrap_or(OsStr::new("")),
				_ => return false,
			});
			if it.peek().is_some() {
				s.push("\n");
			}
		}

		futures::executor::block_on(external::clipboard_set(s)).ok();
		false
	}

	pub fn find(&mut self, query: Option<&str>, prev: bool, case: FinderCase) -> bool {
		if let Some(query) = query {
			let Ok(finder) = Finder::new(query, case) else {
				return false;
			};

			let step = if prev {
				finder.prev(&self.current.files, self.current.cursor, true)
			} else {
				finder.next(&self.current.files, self.current.cursor, true)
			};

			if let Some(step) = step {
				self.arrow(step.into());
			}

			self.finder = Some(finder);
			return true;
		}

		tokio::spawn(async move {
			let rx = emit!(Input(
				InputOpt::top(if prev { "Find previous:" } else { "Find next:" }).with_realtime()
			));

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(Ok(s)) | Some(Err(InputError::Typed(s))) = rx.next().await {
				emit!(Call(
					Exec::call("find", vec![s])
						.with_bool("previous", prev)
						.with_bool("smart", case == FinderCase::Smart)
						.with_bool("insensitive", case == FinderCase::Insensitive)
						.vec(),
					KeymapLayer::Manager
				));
			}
		});
		false
	}

	pub fn find_arrow(&mut self, prev: bool) -> bool {
		let Some(finder) = &mut self.finder else {
			return false;
		};

		let b = finder.catchup(&self.current.files);
		let step = if prev {
			finder.prev(&self.current.files, self.current.cursor, false)
		} else {
			finder.next(&self.current.files, self.current.cursor, false)
		};

		b | step.is_some_and(|s| self.arrow(s.into()))
	}

	pub fn search(&mut self, grep: bool) -> bool {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}

		let mut cwd = self.current.cwd.clone();
		let hidden = self.show_hidden;

		self.search = Some(tokio::spawn(async move {
			let Some(Ok(subject)) = emit!(Input(InputOpt::top("Search:"))).recv().await else {
				bail!("canceled")
			};

			cwd = cwd.into_search(subject.clone());
			let rx = if grep {
				external::rg(external::RgOpt { cwd: cwd.clone(), hidden, subject })
			} else {
				external::fd(external::FdOpt { cwd: cwd.clone(), hidden, glob: false, subject })
			}?;

			let rx = UnboundedReceiverStream::new(rx).chunks_timeout(1000, Duration::from_millis(300));
			pin!(rx);

			let ticket = FilesOp::prepare(&cwd);
			let mut first = true;
			while let Some(chunk) = rx.next().await {
				if first {
					emit!(Cd(cwd.clone()));
					first = false;
				}
				emit!(Files(FilesOp::Part(cwd.clone(), ticket, chunk)));
			}
			Ok(())
		}));
		true
	}

	pub fn search_stop(&mut self) -> bool {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}
		if self.current.cwd.is_search() {
			self.preview_reset_image();

			let rep = self.history_new(&self.current.cwd.to_regular());
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
			.map(|f| (f.url().as_os_str().to_owned(), Default::default()))
			.collect();

		let mut exec = exec.to_owned();
		tokio::spawn(async move {
			if !confirm || exec.is_empty() {
				let mut result = emit!(Input(
					InputOpt::top(if block { "Shell (block):" } else { "Shell:" })
						.with_value(&exec)
						.with_highlight()
				));
				match result.recv().await {
					Some(Ok(e)) => exec = e,
					_ => return,
				}
			}

			emit!(Open(
				selected,
				Some(Opener { exec, block, orphan: false, display_name: Default::default(), spread: true })
			));
		});

		false
	}

	pub fn update_peek(&mut self, max: usize, url: Url) -> bool {
		let Some(hovered) = self.current.hovered() else {
			return false;
		};

		if &url != hovered.url() {
			return false;
		}

		self.preview.arrow_max(max)
	}

	pub fn update_preview(&mut self, lock: PreviewLock) -> bool {
		let Some(hovered) = self.current.hovered().map(|h| h.url()) else {
			return self.preview_reset();
		};

		if lock.url != *hovered {
			return false;
		}

		self.preview.lock = Some(lock);
		true
	}
}

impl Tab {
	// --- Mode
	#[inline]
	pub fn in_selecting(&self) -> bool { self.mode.is_visual() || self.current.files.has_selected() }

	// --- Current
	pub fn selected(&self) -> Vec<&File> {
		let pending = self.mode.visual().map(|(_, p)| Cow::Borrowed(p)).unwrap_or_default();
		let selected = self.current.files.selected(&pending, self.mode.is_unset());

		if selected.is_empty() {
			self.current.hovered().map(|h| vec![h]).unwrap_or_default()
		} else {
			selected
		}
	}

	// --- History
	#[inline]
	pub fn history(&self, url: &Url) -> Option<&Folder> { self.history.get(url) }

	#[inline]
	pub fn history_new(&mut self, url: &Url) -> Folder {
		self.history.remove(url).unwrap_or_else(|| Folder::from(url))
	}

	// --- Preview
	#[inline]
	pub fn preview(&self) -> &Preview { &self.preview }

	#[inline]
	pub fn preview_reset(&mut self) -> bool { self.preview.reset(|_| true) }

	#[inline]
	pub fn preview_reset_image(&mut self) -> bool { self.preview.reset(|l| l.is_image()) }

	#[inline]
	pub fn preview_arrow(&mut self, step: isize) -> bool { self.preview.arrow(step) }

	// --- Finder
	#[inline]
	pub fn finder(&self) -> Option<&Finder> { self.finder.as_ref() }

	// --- Sorter
	pub fn set_sorter(&mut self, sorter: FilesSorter) -> bool {
		if sorter == self.sorter {
			return false;
		}

		self.sorter = sorter;
		self.apply_files_attrs(false)
	}

	// --- Show hidden
	pub fn set_show_hidden(&mut self, state: Option<bool>) -> bool {
		let state = state.unwrap_or(!self.show_hidden);
		if state == self.show_hidden {
			return false;
		}

		self.show_hidden = state;
		if self.apply_files_attrs(false) {
			emit!(Peek);
			return true;
		}
		false
	}

	pub fn apply_files_attrs(&mut self, just_preview: bool) -> bool {
		let mut b = false;
		if let Some(f) =
			self.current.hovered().filter(|h| h.is_dir()).and_then(|h| self.history.get_mut(h.url()))
		{
			b |= f.files.set_show_hidden(self.show_hidden);
			b |= f.files.set_sorter(self.sorter);
		}

		if just_preview {
			return b;
		}

		let hovered = self.current.hovered().map(|h| h.url_owned());
		b |= self.current.files.set_show_hidden(self.show_hidden);
		b |= self.current.files.set_sorter(self.sorter);

		if let Some(parent) = self.parent.as_mut() {
			b |= parent.files.set_show_hidden(self.show_hidden);
			b |= parent.files.set_sorter(self.sorter);
		}

		self.current.repos(hovered);
		b
	}
}
