use std::{collections::{BTreeMap, BTreeSet, HashMap, HashSet}, env, ffi::OsStr, io::{stdout, Write}, mem, os::unix::prelude::OsStrExt, path::{Path, PathBuf}};

use anyhow::{bail, Error, Result};
use config::{open::Opener, BOOT, OPEN};
use shared::{in_same_root, Defer, Term, MIME_DIR};
use tokio::{fs::{self, OpenOptions}, io::{AsyncReadExt, AsyncWriteExt}};

use super::{PreviewData, Tab, Tabs, Watcher};
use crate::{emit, external, files::{File, FilesOp}, input::InputOpt, manager::Folder, select::SelectOpt, tasks::Tasks};

pub struct Manager {
	tabs:   Tabs,
	yanked: (bool, HashSet<PathBuf>),

	watcher:      Watcher,
	pub mimetype: HashMap<PathBuf, String>,
}

impl Manager {
	pub fn make() -> Self {
		Self {
			tabs:   Tabs::make(),
			yanked: Default::default(),

			watcher:  Watcher::start(),
			mimetype: Default::default(),
		}
	}

	pub fn refresh(&mut self) {
		env::set_current_dir(self.cwd()).ok();

		self.watcher.trigger(self.cwd());
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
			if let Some(ref h) = tab.current.hovered {
				if h.meta.is_dir() {
					to_watch.insert(h.path());
				}
			}
		}
		self.watcher.watch(to_watch);
	}

	pub fn preview(&mut self, show_image: bool) -> bool {
		let hovered = if let Some(h) = self.hovered() {
			h.clone()
		} else {
			return self.active_mut().preview.reset();
		};

		if !show_image {
			self.active_mut().preview_reset_image();
		}

		if hovered.meta.is_dir() {
			self.active_mut().preview.go(&hovered.path, MIME_DIR, show_image);
			if self.active().history(&hovered.path).is_some() {
				emit!(Preview(hovered.path, MIME_DIR.to_owned(), PreviewData::Folder));
			}
		} else if let Some(mime) = self.mimetype.get(&hovered.path).cloned() {
			self.active_mut().preview.go(&hovered.path, &mime, show_image);
		} else {
			tokio::spawn(async move {
				if let Ok(mimes) = external::file(&[hovered.path]).await {
					emit!(Mimetype(mimes));
				}
			});
		}
		false
	}

	pub fn yank(&mut self, cut: bool) -> bool {
		let selected = self.selected().into_iter().map(|f| f.path()).collect::<Vec<_>>();
		self.yanked.0 = cut;
		self.yanked.1.clear();
		self.yanked.1.extend(selected);
		false
	}

	#[inline]
	pub fn yanked(&self) -> &(bool, HashSet<PathBuf>) { &self.yanked }

	pub fn quit(&self, tasks: &Tasks) -> bool {
		let tasks = tasks.len();
		if tasks == 0 {
			emit!(Quit);
			return false;
		}

		tokio::spawn(async move {
			let result = emit!(Input(InputOpt::top(format!(
				"There are {tasks} tasks running, sure to quit? (y/N)"
			))));

			if let Ok(choice) = result.await {
				if choice.to_lowercase() == "y" {
					emit!(Quit);
				}
			}
		});
		false
	}

	pub fn close(&mut self, tasks: &Tasks) -> bool {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.idx());
		}
		self.quit(tasks)
	}

	pub fn open(&mut self, interactive: bool) -> bool {
		let mut files = self
			.selected()
			.into_iter()
			.map(|f| {
				(
					f.path().into_os_string(),
					if f.meta.is_dir() {
						Some(MIME_DIR.to_owned())
					} else {
						self.mimetype.get(&f.path).cloned()
					},
				)
			})
			.collect::<Vec<_>>();

		if files.is_empty() {
			return false;
		}

		tokio::spawn(async move {
			let todo = files.iter().filter(|(_, m)| m.is_none()).map(|(p, _)| p).collect::<Vec<_>>();
			if let Ok(mut mimes) = external::file(&todo).await {
				files = files
					.into_iter()
					.map(|(p, m)| {
						let mime = m.or_else(|| mimes.remove(&PathBuf::from(&p)));
						(p, mime)
					})
					.collect::<Vec<_>>();
			}

			let files = files.into_iter().filter_map(|(p, m)| m.map(|m| (p, m))).collect::<Vec<_>>();
			if !interactive {
				emit!(Open(files, None));
				return;
			}

			let openers = OPEN.common_openers(&files);
			if openers.is_empty() {
				return;
			}

			let result = emit!(Select(SelectOpt::hovered(
				"Open with:",
				openers.iter().map(|o| o.display_name.clone()).collect()
			)));
			if let Ok(choice) = result.await {
				emit!(Open(files, Some(openers[choice].clone())));
			}
		});
		false
	}

	pub fn create(&self) -> bool {
		let cwd = self.cwd().to_owned();
		tokio::spawn(async move {
			let result = emit!(Input(InputOpt::top("Create:")));

			if let Ok(name) = result.await {
				let path = cwd.join(&name);
				let hovered = path.components().take(cwd.components().count() + 1).collect::<PathBuf>();

				if name.ends_with('/') {
					fs::create_dir_all(path).await?;
				} else {
					fs::create_dir_all(path.parent().unwrap()).await.ok();
					fs::File::create(path).await?;
				}

				if let Ok(file) = File::from(&hovered).await {
					emit!(Hover(file));
					emit!(Refresh);
				}
			}
			Ok::<(), Error>(())
		});
		false
	}

	pub fn rename(&self) -> bool {
		if self.in_selecting() {
			return self.bulk_rename();
		}

		let hovered = if let Some(h) = self.hovered() {
			h.path.clone()
		} else {
			return false;
		};

		tokio::spawn(async move {
			let result = emit!(Input(
				InputOpt::hovered("Rename:").with_value(hovered.file_name().unwrap().to_string_lossy())
			));

			if let Ok(new) = result.await {
				let to = hovered.parent().unwrap().join(new);
				fs::rename(&hovered, to).await.ok();
			}
		});
		false
	}

	pub fn bulk_rename(&self) -> bool {
		let mut old: Vec<_> = self.selected().iter().map(|&f| f.path()).collect();
		if old.is_empty() {
			return false;
		}

		let root = in_same_root(&old);
		if let Some(ref root) = root {
			old = old.into_iter().map(|p| p.strip_prefix(root).unwrap().to_owned()).collect();
		}

		let tmp = BOOT.tmpfile();
		tokio::spawn(async move {
			let Some(opener) = OPEN.block_opener("bulk-rename.txt", "text/plain") else {
				bail!("No opener for bulk rename");
			};

			{
				let b = old.iter().map(|o| o.as_os_str()).collect::<Vec<_>>().join(OsStr::new("\n"));
				let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp).await?;
				f.write_all(b.as_bytes()).await?;
			}

			let _guard = BLOCKER.acquire().await.unwrap();
			let _defer = Defer::new(|| Event::Stop(false, None).emit());
			emit!(Stop(true)).await;

			let mut child = external::shell(ShellOpt {
				cmd:   (*opener.exec).into(),
				args:  vec![tmp.to_owned().into()],
				piped: false,
			})?;
			child.wait().await?;

			let new: Vec<_> = fs::read_to_string(tmp).await?.lines().map(|l| l.into()).collect();
			Self::bulk_rename_do(root, old, new).await
		});

		false
	}

	async fn bulk_rename_do(
		root: Option<PathBuf>,
		old: Vec<PathBuf>,
		new: Vec<PathBuf>,
	) -> Result<()> {
		Term::clear()?;
		if old.len() != new.len() {
			println!("Number of old and new differ, press ENTER to exit");
			tokio::io::stdin().read_exact(&mut [0]).await?;
			return Ok(());
		}

		let mut todo = Vec::with_capacity(old.len());
		for (o, n) in old.into_iter().zip(new) {
			if n != o {
				stdout().write_all(o.as_os_str().as_bytes())?;
				stdout().write_all(b" -> ")?;
				stdout().write_all(n.as_os_str().as_bytes())?;
				stdout().write_all(b"\n")?;
				todo.push(if let Some(ref root) = root { (root.join(o), root.join(n)) } else { (o, n) });
			}
		}
		if todo.is_empty() {
			return Ok(());
		} else {
			print!("Continue to rename? (y/N): ");
			stdout().flush()?;
		}

		let mut buf = [0];
		tokio::io::stdin().read_exact(&mut buf).await?;
		if buf[0] != b'y' && buf[0] != b'Y' {
			return Ok(());
		}

		let mut failed = Vec::new();
		for (o, n) in todo {
			if let Err(e) = fs::rename(&o, &n).await {
				failed.push((o, n, e));
			}
		}

		if !failed.is_empty() {
			Term::clear()?;
			println!("Failed to rename:");
			for (o, n, e) in failed {
				stdout().write_all(o.as_os_str().as_bytes())?;
				stdout().write_all(b" -> ")?;
				stdout().write_all(n.as_os_str().as_bytes())?;
				stdout().write_fmt(format_args!(": {e}\n"))?;
			}
			println!("\nPress ENTER to exit");
			tokio::io::stdin().read_exact(&mut [0]).await?;
		}
		Ok(())
	}

	pub fn shell(&self, exec: &str, block: bool, confirm: bool) -> bool {
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
				Default::default(),
				Some(Opener { exec, block, display_name: Default::default(), spread: true })
			));
		});

		false
	}

	pub fn update_read(&mut self, op: FilesOp) -> bool {
		let path = op.path();
		let cwd = self.cwd().to_owned();
		let hovered = self.hovered().map(|h| h.path());

		let mut b = if cwd == path && !self.current().in_search {
			self.current_mut().update(op)
		} else if matches!(self.parent(), Some(p) if p.cwd == path) {
			self.active_mut().parent.as_mut().unwrap().update(op)
		} else {
			self
				.active_mut()
				.history
				.entry(path.to_path_buf())
				.or_insert_with(|| Folder::new(&path))
				.update(op);

			matches!(self.hovered(), Some(h) if h.path == path)
		};

		b |= self.active_mut().parent.as_mut().map_or(false, |p| p.hover(&cwd));
		b |= hovered.as_ref().map_or(false, |h| self.current_mut().hover(h));

		if hovered != self.hovered().map(|h| h.path()) {
			emit!(Hover);
		}
		b
	}

	pub fn update_search(&mut self, op: FilesOp) -> bool {
		let path = op.path();
		if self.current().in_search && self.cwd() == path {
			return self.current_mut().update(op);
		}

		let rep = mem::replace(self.current_mut(), Folder::new_search(&path));
		if !rep.in_search {
			self.active_mut().history.insert(path, rep);
		}
		self.current_mut().update(op);
		true
	}

	pub fn update_ioerr(&mut self, op: FilesOp) -> bool {
		let path = op.path();
		let op = FilesOp::read_empty(&path);

		if path == self.cwd() {
			self.current_mut().update(op);
		} else if matches!(self.parent(), Some(p) if p.cwd == path) {
			self.active_mut().parent.as_mut().unwrap().update(op);
		} else {
			return false;
		}

		self.active_mut().leave();
		true
	}

	pub fn update_mimetype(&mut self, mut mimes: BTreeMap<PathBuf, String>, tasks: &Tasks) -> bool {
		mimes.retain(|f, m| self.mimetype.get(f) != Some(m));
		if mimes.is_empty() {
			return false;
		}

		tasks.precache_image(&mimes);
		tasks.precache_video(&mimes);
		tasks.precache_pdf(&mimes);

		self.mimetype.extend(mimes);
		true
	}

	pub fn update_preview(&mut self, path: PathBuf, mime: String, data: PreviewData) -> bool {
		let hovered = if let Some(ref h) = self.current().hovered {
			h.path()
		} else {
			return self.active_mut().preview.reset();
		};

		if hovered != path {
			return false;
		}

		let preview = &mut self.active_mut().preview;
		preview.lock = Some((path, mime));
		preview.data = data;
		true
	}
}

impl Manager {
	#[inline]
	pub fn cwd(&self) -> &Path { &self.current().cwd }

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
	pub fn parent(&self) -> Option<&Folder> { self.tabs.active().parent.as_ref() }

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.tabs.active().current.hovered.as_ref() }

	pub fn selected(&self) -> Vec<&File> {
		let mode = &self.active().mode;
		let files = &self.current().files;

		let selected: Vec<_> = if !mode.is_visual() {
			files.iter().filter(|(_, f)| f.is_selected).map(|(_, f)| f).collect()
		} else {
			files
				.iter()
				.enumerate()
				.filter(|(i, (_, f))| mode.pending(*i, f.is_selected))
				.map(|(_, (_, f))| f)
				.collect()
		};

		if selected.is_empty() { self.hovered().map(|h| vec![h]).unwrap_or_default() } else { selected }
	}

	#[inline]
	pub fn in_selecting(&self) -> bool {
		self.active().mode.is_visual() || self.current().has_selected()
	}
}
