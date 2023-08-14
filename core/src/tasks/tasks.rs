use std::{collections::{BTreeMap, HashMap, HashSet}, ffi::OsStr, io::{stdout, Write}, path::{Path, PathBuf}, sync::Arc};

use config::{manager::SortBy, open::Opener, OPEN};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use shared::{tty_size, Defer, MimeKind, Term};
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
		(tty_size().ws_row * TASKS_PERCENT / 100).saturating_sub(TASKS_PADDING) as usize
	}

	pub fn toggle(&mut self) -> bool {
		self.visible = !self.visible;
		emit!(Hover); // Show/hide preview for images
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
		running.values().take(Self::limit()).map(|t| t.into()).collect()
	}

	pub fn inspect(&self) -> bool {
		let id = if let Some(id) = self.scheduler.running.read().get_id(self.cursor) {
			id
		} else {
			return false;
		};

		let scheduler = self.scheduler.clone();
		tokio::spawn(async move {
			let _guard = BLOCKER.acquire().await.unwrap();
			let (tx, mut rx) = mpsc::unbounded_channel();

			let buffered = {
				let mut running = scheduler.running.write();
				let task = if let Some(task) = running.get_mut(id) { task } else { return };

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
		if !id.map(|id| self.scheduler.cancel(id)).unwrap_or(false) {
			return false;
		}

		self.next();
		true
	}

	pub fn file_open(&self, targets: &[(impl AsRef<Path>, impl AsRef<str>)]) -> bool {
		let mut openers = BTreeMap::new();
		for (path, mime) in targets {
			if let Some(opener) = OPEN.openers(path, mime).and_then(|o| o.first()) {
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

	pub fn file_cut(&self, src: &HashSet<PathBuf>, dest: PathBuf, force: bool) -> bool {
		for p in src {
			let to = dest.join(p.file_name().unwrap());
			if force && *p == to {
				trace!("file_cut: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_cut(p.clone(), to, force);
			}
		}
		false
	}

	pub fn file_copy(
		&self,
		src: &HashSet<PathBuf>,
		dest: PathBuf,
		force: bool,
		follow: bool,
	) -> bool {
		for p in src {
			let to = dest.join(p.file_name().unwrap());
			if force && *p == to {
				trace!("file_copy: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_copy(p.clone(), to, force, follow);
			}
		}
		false
	}

	pub fn file_remove(&self, targets: Vec<PathBuf>, permanently: bool) -> bool {
		let scheduler = self.scheduler.clone();
		tokio::spawn(async move {
			let result = emit!(Input(InputOpt::hovered("Are you sure delete these files? (y/N)")));

			if let Ok(choice) = result.await {
				if choice.to_lowercase() != "y" {
					return;
				}
				for p in targets {
					if permanently {
						scheduler.file_delete(p);
					} else {
						scheduler.file_trash(p);
					}
				}
			}
		});
		false
	}

	#[inline]
	pub fn precache_size(&self, targets: &Files) -> bool {
		if targets.sort.by != SortBy::Size {
			return false;
		}

		let targets = targets
			.iter()
			.filter(|(_, f)| f.meta.is_dir() && f.length.is_none())
			.map(|(p, _)| p.clone())
			.collect::<Vec<_>>();

		if !targets.is_empty() {
			self.scheduler.precache_size(targets);
		}

		false
	}

	#[inline]
	pub fn precache_mime(&self, targets: Vec<&File>, mimetype: &HashMap<PathBuf, String>) -> bool {
		let targets = targets
			.into_iter()
			.filter(|f| f.meta.is_file() && !mimetype.contains_key(&f.path))
			.map(|f| f.path.clone())
			.collect::<Vec<_>>();

		if !targets.is_empty() {
			self.scheduler.precache_mime(targets);
		}
		false
	}

	pub fn precache_image(&self, mimetype: &BTreeMap<PathBuf, String>) -> bool {
		let targets = mimetype
			.iter()
			.filter(|(_, m)| MimeKind::new(m) == MimeKind::Image)
			.map(|(p, _)| p.clone())
			.collect::<Vec<_>>();

		if !targets.is_empty() {
			self.scheduler.precache_image(targets);
		}
		false
	}

	pub fn precache_video(&self, mimetype: &BTreeMap<PathBuf, String>) -> bool {
		let targets = mimetype
			.iter()
			.filter(|(_, m)| MimeKind::new(m) == MimeKind::Video)
			.map(|(p, _)| p.clone())
			.collect::<Vec<_>>();

		if !targets.is_empty() {
			self.scheduler.precache_video(targets);
		}
		false
	}

	pub fn precache_pdf(&self, mimetype: &BTreeMap<PathBuf, String>) -> bool {
		let targets = mimetype
			.iter()
			.filter(|(_, m)| MimeKind::new(m) == MimeKind::PDF)
			.map(|(p, _)| p.clone())
			.collect::<Vec<_>>();

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
