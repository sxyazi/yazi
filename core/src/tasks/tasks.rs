use std::{collections::{BTreeMap, HashMap, HashSet}, ffi::OsStr, io::{stdout, Write}, path::Path, sync::Arc};

use config::{manager::SortBy, open::Opener, OPEN};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use shared::{Defer, MimeKind, Term, Url};
use tokio::{io::{stdin, AsyncReadExt}, select, sync::mpsc, time};
use tracing::trace;

use super::{task::TaskSummary, Scheduler, TASKS_PADDING, TASKS_PERCENT};
use crate::{emit, files::{File, Files}, input::InputOpt, Event, BLOCKER};

pub struct Tasks {
	scheduler: Arc<Scheduler>,

	pub visible:  bool,
	pub cursor:   usize,
	pub progress: (u8, u32),
}

impl Tasks {
	pub fn start() -> Self {
		Self {
			scheduler: Arc::new(Scheduler::start()),
			visible:   false,
			cursor:    0,
			progress:  (100, 0),
		}
	}

	#[inline]
	pub fn limit() -> usize {
		(Term::size().rows * TASKS_PERCENT / 100).saturating_sub(TASKS_PADDING) as usize
	}

	pub fn toggle(&mut self) -> bool {
		self.visible = !self.visible;
		emit!(Peek); // Show/hide preview for images
		true
	}

	#[allow(clippy::should_implement_trait)]
	pub fn next(&mut self) -> bool {
		let limit = Self::limit().min(self.len());

		let old = self.cursor;
		self.cursor = limit.saturating_sub(1).min(self.cursor + 1);

		old != self.cursor
	}

	pub fn prev(&mut self) -> bool {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(1);
		old != self.cursor
	}

	pub fn paginate(&self) -> Vec<TaskSummary> {
		let running = self.scheduler.running.read();
		running.values().take(Self::limit()).map(Into::into).collect()
	}

	pub fn inspect(&self) -> bool {
		let Some(id) = self.scheduler.running.read().get_id(self.cursor) else {
			return false;
		};

		let scheduler = self.scheduler.clone();
		tokio::spawn(async move {
			let _guard = BLOCKER.acquire().await.unwrap();
			let (tx, mut rx) = mpsc::unbounded_channel();

			let buffered = {
				let mut running = scheduler.running.write();
				let Some(task) = running.get_mut(id) else { return };

				task.logger = Some(tx);
				task.logs.clone()
			};

			emit!(Stop(true)).await;
			let _defer = Defer::new(|| {
				disable_raw_mode().ok();
				Event::Stop(false, None).emit();
			});

			Term::clear(&mut stdout()).ok();
			stdout().write_all(buffered.as_bytes()).ok();
			enable_raw_mode().ok();

			let mut stdin = stdin();
			let mut quit = [0; 10];
			loop {
				select! {
					Some(line) = rx.recv() => {
						let mut stdout = stdout().lock();
						stdout.write_all(line.as_bytes()).ok();
						stdout.write_all(b"\r\n").ok();
					}
					_ = time::sleep(time::Duration::from_millis(100)) => {
						if scheduler.running.read().get(id).is_none() {
							stdout().write_all(b"Task finished, press `q` to quit\r\n").ok();
							break;
						}
					},
					Ok(_) = stdin.read(&mut quit) => {
						if quit[0] == b'q' {
							break;
						}
					}
				}
			}

			if let Some(task) = scheduler.running.write().get_mut(id) {
				task.logger = None;
			}
			while quit[0] != b'q' {
				stdin.read(&mut quit).await.ok();
			}
		});
		false
	}

	pub fn cancel(&mut self) -> bool {
		let id = self.scheduler.running.read().get_id(self.cursor);
		if id.map(|id| self.scheduler.cancel(id)) != Some(true) {
			return false;
		}

		let len = self.scheduler.running.read().len();
		self.cursor = self.cursor.min(len.saturating_sub(1));
		true
	}

	pub fn file_open(&self, targets: &[(impl AsRef<Path>, impl AsRef<str>)]) -> bool {
		let mut openers = BTreeMap::new();
		for (path, mime) in targets {
			if let Some(opener) = OPEN.openers(path, mime).and_then(|o| o.first().copied()) {
				openers.entry(opener).or_insert_with(Vec::new).push(path.as_ref().as_os_str());
			}
		}
		for (opener, args) in openers {
			self.file_open_with(opener, &args);
		}
		false
	}

	pub fn file_open_with(&self, opener: &Opener, args: &[impl AsRef<OsStr>]) -> bool {
		if opener.spread {
			self.scheduler.process_open(opener, args);
			return false;
		}
		for target in args {
			self.scheduler.process_open(opener, &[target]);
		}
		false
	}

	pub fn file_cut(&self, src: &HashSet<Url>, dest: &Url, force: bool) -> bool {
		for u in src {
			let to = dest.join(u.file_name().unwrap());
			if force && u == &to {
				trace!("file_cut: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_cut(u.clone(), to, force);
			}
		}
		false
	}

	pub fn file_copy(&self, src: &HashSet<Url>, dest: &Url, force: bool) -> bool {
		for u in src {
			let to = dest.join(u.file_name().unwrap());
			if force && u == &to {
				trace!("file_copy: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_copy(u.clone(), to, force);
			}
		}
		false
	}

	pub fn file_link(&self, src: &HashSet<Url>, dest: &Url, relative: bool, force: bool) -> bool {
		for u in src {
			let to = dest.join(u.file_name().unwrap());
			if force && *u == to {
				trace!("file_link: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_link(u.clone(), to, relative, force);
			}
		}
		false
	}

	pub fn file_remove(&self, targets: Vec<Url>, force: bool, permanently: bool) -> bool {
		if force {
			for u in targets {
				if permanently {
					self.scheduler.file_delete(u);
				} else {
					self.scheduler.file_trash(u);
				}
			}
			return false;
		}

		let scheduler = self.scheduler.clone();
		tokio::spawn(async move {
			let s = if targets.len() > 1 { "s" } else { "" };
			let mut result = emit!(Input(InputOpt::hovered(if permanently {
				format!("Delete selected file{s} permanently? (y/N)")
			} else {
				format!("Move selected file{s} to trash? (y/N)")
			})));

			if let Some(Ok(choice)) = result.recv().await {
				if choice != "y" && choice != "Y" {
					return;
				}
				for u in targets {
					if permanently {
						scheduler.file_delete(u);
					} else {
						scheduler.file_trash(u);
					}
				}
			}
		});
		false
	}

	#[inline]
	pub fn precache_size(&self, targets: &Files) -> bool {
		if targets.sorter().by != SortBy::Size {
			return false;
		}

		let targets: Vec<_> = targets
			.iter()
			.filter(|f| f.is_dir() && targets.size(f.url()).is_none())
			.map(|f| f.url())
			.collect();

		if !targets.is_empty() {
			self.scheduler.precache_size(targets);
		}

		false
	}

	#[inline]
	pub fn precache_mime(&self, targets: &[File], mimetype: &HashMap<Url, String>) -> bool {
		let targets: Vec<_> = targets
			.iter()
			.filter(|f| f.is_file() && !mimetype.contains_key(f.url()))
			.map(|f| f.url_owned())
			.collect();

		if !targets.is_empty() {
			self.scheduler.precache_mime(targets);
		}
		false
	}

	pub fn precache_image(&self, mimetype: &BTreeMap<Url, String>) -> bool {
		let targets: Vec<_> = mimetype
			.iter()
			.filter(|(_, m)| MimeKind::new(m) == MimeKind::Image)
			.map(|(u, _)| u.clone())
			.collect();

		if !targets.is_empty() {
			self.scheduler.precache_image(targets);
		}
		false
	}

	pub fn precache_video(&self, mimetype: &BTreeMap<Url, String>) -> bool {
		let targets: Vec<_> = mimetype
			.iter()
			.filter(|(_, m)| MimeKind::new(m) == MimeKind::Video)
			.map(|(u, _)| u.clone())
			.collect();

		if !targets.is_empty() {
			self.scheduler.precache_video(targets);
		}
		false
	}

	pub fn precache_pdf(&self, mimetype: &BTreeMap<Url, String>) -> bool {
		let targets: Vec<_> = mimetype
			.iter()
			.filter(|(_, m)| MimeKind::new(m) == MimeKind::PDF)
			.map(|(u, _)| u.clone())
			.collect();

		if !targets.is_empty() {
			self.scheduler.precache_pdf(targets);
		}
		false
	}
}

impl Tasks {
	#[inline]
	pub fn len(&self) -> usize { self.scheduler.running.read().len() }
}
