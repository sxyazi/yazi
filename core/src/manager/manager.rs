use std::{collections::{BTreeMap, BTreeSet, HashMap, HashSet}, env, ffi::OsStr, io::{stdout, BufWriter, Write}, mem, path::{Path, PathBuf}};

use anyhow::{anyhow, bail, Error, Result};
use config::{BOOT, OPEN};
use shared::{max_common_root, Defer, Term, MIME_DIR};
use tokio::{fs::{self, OpenOptions}, io::{stdin, AsyncReadExt, AsyncWriteExt}};

use super::{Tab, Tabs, Watcher};
use crate::{emit, external::{self, ShellOpt}, files::{File, FilesOp}, input::InputOpt, manager::Folder, select::SelectOpt, tasks::Tasks, Event, BLOCKER};

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

		if let Some(f) = self.parent() {
			self.watcher.trigger_dirs(&[self.cwd(), &f.cwd]);
		} else {
			self.watcher.trigger_dirs(&[self.cwd()]);
		}
		emit!(Hover);

		let mut to_watch = BTreeSet::new();
		for tab in self.tabs.iter() {
			to_watch.insert(&tab.current.cwd);
			if let Some(ref h) = tab.current.hovered {
				if h.is_dir() {
					to_watch.insert(h.path());
				}
			}
			if let Some(ref p) = tab.parent {
				to_watch.insert(&p.cwd);
			}
		}
		self.watcher.watch(to_watch);
	}

	pub fn peek(&mut self, sequent: bool, show_image: bool) -> bool {
		let Some(hovered) = self.hovered().cloned() else {
			return self.active_mut().preview.reset();
		};

		if !show_image {
			self.active_mut().preview_reset_image();
		}

		let mime = if hovered.is_dir() {
			MIME_DIR.to_owned()
		} else if let Some(m) = self.mimetype.get(hovered.path()).cloned() {
			m
		} else {
			tokio::spawn(async move {
				if let Ok(mimes) = external::file(&[hovered.path()]).await {
					emit!(Mimetype(mimes));
				}
			});
			return false;
		};

		if sequent {
			self.active_mut().preview.sequent(hovered.path(), &mime, show_image);
		} else {
			self.active_mut().preview.go(hovered.path(), &mime, show_image);
		}
		false
	}

	pub fn yank(&mut self, cut: bool) -> bool {
		self.yanked.0 = cut;
		self.yanked.1 = self.selected().into_iter().map(|f| f.path().clone()).collect();
		false
	}

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
		let mut files: Vec<_> = self
			.selected()
			.into_iter()
			.map(|f| {
				(
					f.path().as_os_str().to_owned(),
					if f.is_dir() { Some(MIME_DIR.to_owned()) } else { self.mimetype.get(f.path()).cloned() },
				)
			})
			.collect();

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
		if self.active().in_selecting() {
			return self.bulk_rename();
		}

		let Some(hovered) = self.hovered().map(|h| h.path().clone()) else {
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
		let old: Vec<_> = self.selected().into_iter().map(|f| f.path()).collect();

		let root = max_common_root(&old);
		let old: Vec<_> = old.into_iter().map(|p| p.strip_prefix(&root).unwrap().to_owned()).collect();

		let tmp = BOOT.tmpfile("bulk");
		tokio::spawn(async move {
			let Some(opener) = OPEN.block_opener("bulk.txt", "text/plain") else {
				bail!("No opener for bulk rename");
			};

			{
				let s = old.iter().map(|o| o.as_os_str()).collect::<Vec<_>>().join(OsStr::new("\n"));
				let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp).await?;
				#[cfg(target_os = "windows")]
				{
					f.write_all(s.to_string_lossy().as_bytes()).await?;
				}
				#[cfg(not(target_os = "windows"))]
				{
					use std::os::unix::ffi::OsStrExt;
					f.write_all(s.as_bytes()).await?;
				}
			}

			let _guard = BLOCKER.acquire().await.unwrap();
			let _defer = Defer::new(|| {
				Event::Stop(false, None).emit();
				tokio::spawn(fs::remove_file(tmp.clone()))
			});
			emit!(Stop(true)).await;

			let mut child = external::shell(ShellOpt {
				cmd:   (*opener.exec).into(),
				args:  vec![tmp.to_owned().into()],
				piped: false,
			})?;
			child.wait().await?;

			let new: Vec<_> = fs::read_to_string(&tmp).await?.lines().map(|l| l.into()).collect();
			Self::bulk_rename_do(root, old, new).await
		});

		false
	}

	async fn bulk_rename_do(root: PathBuf, old: Vec<PathBuf>, new: Vec<PathBuf>) -> Result<()> {
		Term::clear(&mut stdout())?;
		if old.len() != new.len() {
			println!("Number of old and new differ, press ENTER to exit");
			stdin().read_exact(&mut [0]).await?;
			return Ok(());
		}

		let todo: Vec<_> = old.into_iter().zip(new).filter(|(o, n)| o != n).collect();
		if todo.is_empty() {
			return Ok(());
		}

		{
			let mut stdout = BufWriter::new(stdout().lock());
			for (o, n) in &todo {
				writeln!(stdout, "{} -> {}", o.display(), n.display())?;
			}
			write!(stdout, "Continue to rename? (y/N): ")?;
			stdout.flush()?;
		}

		let mut buf = [0; 10];
		stdin().read(&mut buf).await.ok();
		if buf[0] != b'y' && buf[0] != b'Y' {
			return Ok(());
		}

		let mut failed = Vec::new();
		for (o, n) in todo {
			if fs::metadata(&n).await.is_ok() {
				failed.push((o, n, anyhow!("Destination already exists")));
				continue;
			}
			if let Err(e) = fs::rename(root.join(&o), root.join(&n)).await {
				failed.push((o, n, e.into()));
			}
		}
		if failed.is_empty() {
			return Ok(());
		}

		Term::clear(&mut stdout())?;
		{
			let mut stdout = BufWriter::new(stdout().lock());
			writeln!(stdout, "Failed to rename:")?;
			for (o, n, e) in failed {
				writeln!(stdout, "{} -> {}: {e}", o.display(), n.display())?;
			}
			writeln!(stdout, "\nPress ENTER to exit")?;
			stdout.flush()?;
		}

		stdin().read_exact(&mut [0]).await?;
		Ok(())
	}

	pub fn update_read(&mut self, op: FilesOp) -> bool {
		let path = op.path();
		let cwd = self.cwd().to_owned();
		let hovered = self.hovered().map(|h| h.path().clone());

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

			matches!(self.hovered(), Some(h) if h.path() == &path)
		};

		b |= self.active_mut().parent.as_mut().map_or(false, |p| p.hover(&cwd));
		b |= hovered.as_ref().map_or(false, |h| self.current_mut().hover(h));

		if hovered.as_ref() != self.hovered().map(|h| h.path()) {
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

	pub fn update_hover(&mut self, file: Option<File>) -> bool {
		let b = file.map(|f| self.current_mut().hover_force(f)).unwrap_or(false);
		let Some(hovered) = self.hovered() else {
			return b;
		};

		if hovered.is_dir() {
			self.watcher.trigger_dirs(&[hovered.path()]);
		}
		b
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

	#[inline]
	pub fn selected(&self) -> Vec<&File> { self.tabs.active().selected() }

	#[inline]
	pub fn yanked(&self) -> &(bool, HashSet<PathBuf>) { &self.yanked }
}
