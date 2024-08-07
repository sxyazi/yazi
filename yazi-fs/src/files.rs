use std::{collections::{HashMap, HashSet}, mem, ops::Deref, sync::atomic::Ordering};

use tokio::{fs::{self, DirEntry}, select, sync::mpsc::{self, UnboundedReceiver}};
use yazi_config::{manager::SortBy, MANAGER};
use yazi_shared::fs::{maybe_exists, Cha, File, FilesOp, Url, FILES_TICKET};

use super::{FilesSorter, Filter};

pub struct Files {
	hidden:       Vec<File>,
	items:        Vec<File>,
	ticket:       u64,
	version:      u64,
	pub revision: u64,

	pub sizes: HashMap<Url, u64>,

	sorter:      FilesSorter,
	filter:      Option<Filter>,
	show_hidden: bool,
}

impl Default for Files {
	fn default() -> Self {
		Self {
			items:    Default::default(),
			hidden:   Default::default(),
			ticket:   Default::default(),
			version:  Default::default(),
			revision: Default::default(),

			sizes: Default::default(),

			sorter:      Default::default(),
			filter:      Default::default(),
			show_hidden: MANAGER.show_hidden,
		}
	}
}

impl Deref for Files {
	type Target = Vec<File>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl Files {
	pub async fn from_dir(url: &Url) -> std::io::Result<UnboundedReceiver<File>> {
		let mut it = fs::read_dir(url).await?;
		let (tx, rx) = mpsc::unbounded_channel();

		tokio::spawn(async move {
			while let Ok(Some(item)) = it.next_entry().await {
				select! {
					_ = tx.closed() => break,
					result = item.metadata() => {
						let url = Url::from(item.path());
						_ = tx.send(match result {
							Ok(meta) => File::from_meta(url, meta).await,
							Err(_) => File::from_dummy(url, item.file_type().await.ok())
						});
					}
				}
			}
		});
		Ok(rx)
	}

	pub async fn from_dir_bulk(url: &Url) -> std::io::Result<Vec<File>> {
		let mut it = fs::read_dir(url).await?;
		let mut items = Vec::with_capacity(5000);
		while let Ok(Some(item)) = it.next_entry().await {
			items.push(item);
		}

		let (first, rest) = items.split_at(items.len() / 3);
		let (second, third) = rest.split_at(items.len() / 3);
		async fn go(entities: &[DirEntry]) -> Vec<File> {
			let mut files = Vec::with_capacity(entities.len() / 3 + 1);
			for entry in entities {
				let url = Url::from(entry.path());
				files.push(match entry.metadata().await {
					Ok(meta) => File::from_meta(url, meta).await,
					Err(_) => File::from_dummy(url, entry.file_type().await.ok()),
				});
			}
			files
		}

		Ok(
			futures::future::join_all([go(first), go(second), go(third)])
				.await
				.into_iter()
				.flatten()
				.collect(),
		)
	}

	pub async fn assert_stale(url: &Url, cha: Cha) -> Option<Cha> {
		match fs::metadata(url).await.map(Cha::from) {
			Ok(c) if !c.is_dir() => {
				// FIXME: use `ErrorKind::NotADirectory` instead once it gets stabilized
				FilesOp::IOErr(url.clone(), std::io::ErrorKind::AlreadyExists).emit();
			}
			Ok(c) if c.hits(cha) => {}
			Ok(c) => return Some(c),
			Err(e) => {
				if maybe_exists(url).await {
					FilesOp::IOErr(url.clone(), e.kind()).emit();
				} else if let Some(p) = url.parent_url() {
					FilesOp::Deleting(p, vec![url.clone()]).emit();
				}
			}
		}
		None
	}
}

impl Files {
	pub fn update_full(&mut self, files: Vec<File>) {
		self.ticket = FILES_TICKET.fetch_add(1, Ordering::Relaxed);

		(self.hidden, self.items) = self.split_files(files);
		if !self.items.is_empty() {
			self.revision += 1;
		}
	}

	pub fn update_part(&mut self, files: Vec<File>, ticket: u64) {
		if !files.is_empty() {
			if ticket != self.ticket {
				return;
			}

			let (hidden, items) = self.split_files(files);
			if !items.is_empty() {
				self.revision += 1;
			}

			self.hidden.extend(hidden);
			self.items.extend(items);
			return;
		}

		self.ticket = ticket;
		self.hidden.clear();
		if !self.items.is_empty() {
			self.revision += 1;
			self.items.clear();
		}
	}

	pub fn update_size(&mut self, sizes: HashMap<Url, u64>) {
		if sizes.is_empty() {
			return;
		}

		if self.sorter.by == SortBy::Size {
			self.revision += 1;
		}
		self.sizes.extend(sizes);
	}

	pub fn update_ioerr(&mut self) {
		self.ticket = FILES_TICKET.fetch_add(1, Ordering::Relaxed);
		self.hidden.clear();
		self.items.clear();
	}

	pub fn update_creating(&mut self, files: Vec<File>) {
		if files.is_empty() {
			return;
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut todo: HashMap<_, _> = $src.into_iter().map(|f| (f.url(), f)).collect();
				for f in &$dist {
					if todo.remove(&f.url).is_some() && todo.is_empty() {
						break;
					}
				}
				if !todo.is_empty() {
					self.revision += $inc;
					$dist.extend(todo.into_values());
				}
			};
		}

		let (hidden, items) = self.split_files(files);
		if !items.is_empty() {
			go!(self.items, items, 1);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden, 0);
		}
	}

	#[cfg(unix)]
	pub fn update_deleting(&mut self, urls: Vec<Url>) {
		if urls.is_empty() {
			return;
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut todo: HashSet<_> = $src.into_iter().collect();
				let len = $dist.len();

				$dist.retain(|f| !todo.remove(&f.url));
				if $dist.len() != len {
					self.revision += $inc;
				}
			};
		}

		let (hidden, items) = if let Some(filter) = &self.filter {
			urls.into_iter().partition(|u| {
				(!self.show_hidden && u.is_hidden()) || !u.file_name().is_some_and(|s| filter.matches(s))
			})
		} else if self.show_hidden {
			(vec![], urls)
		} else {
			urls.into_iter().partition(|u| u.is_hidden())
		};

		if !items.is_empty() {
			go!(self.items, items, 1);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden, 0);
		}
	}

	#[cfg(windows)]
	pub fn update_deleting(&mut self, urls: Vec<Url>) {
		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let len = $dist.len();

				$dist.retain(|f| !$src.remove(&f.url));
				if $dist.len() != len {
					self.revision += $inc;
				}
			};
		}

		let mut urls: HashSet<_> = urls.into_iter().collect();
		if !urls.is_empty() {
			go!(self.items, urls, 1);
		}
		if !urls.is_empty() {
			go!(self.hidden, urls, 0);
		}
	}

	pub fn update_updating(
		&mut self,
		files: HashMap<Url, File>,
	) -> (HashMap<Url, File>, HashMap<Url, File>) {
		if files.is_empty() {
			return Default::default();
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut b = true;
				for i in 0..$dist.len() {
					if let Some(f) = $src.remove(&$dist[i].url) {
						b &= $dist[i].cha.hits(f.cha);
						$dist[i] = f;

						if $src.is_empty() {
							break;
						}
					}
				}
				self.revision += if b { 0 } else { $inc };
			};
		}

		let (mut hidden, mut items) = if let Some(filter) = &self.filter {
			files.into_iter().partition(|(_, f)| {
				(f.is_hidden() && !self.show_hidden)
					|| !f.url.file_name().is_some_and(|s| filter.matches(s))
			})
		} else if self.show_hidden {
			(HashMap::new(), files)
		} else {
			files.into_iter().partition(|(_, f)| f.is_hidden())
		};

		if !items.is_empty() {
			go!(self.items, items, 1);
		}
		if !hidden.is_empty() {
			go!(self.hidden, hidden, 0);
		}
		(hidden, items)
	}

	pub fn update_upserting(&mut self, files: HashMap<Url, File>) {
		if files.is_empty() {
			return;
		}

		let (hidden, items) = self.update_updating(files);
		if hidden.is_empty() && items.is_empty() {
			return;
		}

		if !hidden.is_empty() {
			self.hidden.extend(hidden.into_values());
		}
		if !items.is_empty() {
			self.revision += 1;
			self.items.extend(items.into_values());
		}
	}

	pub fn catchup_revision(&mut self) -> bool {
		if self.version == self.revision {
			return false;
		}

		self.version = self.revision;
		self.sorter.sort(&mut self.items, &self.sizes);
		true
	}

	fn split_files(&self, files: impl IntoIterator<Item = File>) -> (Vec<File>, Vec<File>) {
		if let Some(filter) = &self.filter {
			files.into_iter().partition(|f| {
				(f.is_hidden() && !self.show_hidden)
					|| !f.url.file_name().is_some_and(|s| filter.matches(s))
			})
		} else if self.show_hidden {
			(vec![], files.into_iter().collect())
		} else {
			files.into_iter().partition(|f| f.is_hidden())
		}
	}
}

impl Files {
	// --- Items
	#[inline]
	pub fn position(&self, url: &Url) -> Option<usize> { self.iter().position(|f| f.url == *url) }

	// --- Ticket
	#[inline]
	pub fn ticket(&self) -> u64 { self.ticket }

	// --- Sorter
	#[inline]
	pub fn sorter(&self) -> &FilesSorter { &self.sorter }

	pub fn set_sorter(&mut self, sorter: FilesSorter) {
		if self.sorter != sorter {
			self.sorter = sorter;
			self.revision += 1;
		}
	}

	// --- Filter
	#[inline]
	pub fn filter(&self) -> Option<&Filter> { self.filter.as_ref() }

	pub fn set_filter(&mut self, filter: Option<Filter>) -> bool {
		if self.filter == filter {
			return false;
		}

		self.filter = filter;
		if self.filter.is_none() {
			let take = mem::take(&mut self.hidden);
			let (hidden, items) = self.split_files(take);

			self.hidden = hidden;
			if !items.is_empty() {
				self.items.extend(items);
				self.sorter.sort(&mut self.items, &self.sizes);
			}
			return true;
		}

		let it = mem::take(&mut self.items).into_iter().chain(mem::take(&mut self.hidden));
		(self.hidden, self.items) = self.split_files(it);
		self.sorter.sort(&mut self.items, &self.sizes);
		true
	}

	// --- Show hidden
	pub fn set_show_hidden(&mut self, state: bool) {
		if self.show_hidden == state {
			return;
		}

		self.show_hidden = state;
		if self.show_hidden && self.hidden.is_empty() {
			return;
		} else if !self.show_hidden && self.items.is_empty() {
			return;
		}

		let take =
			if self.show_hidden { mem::take(&mut self.hidden) } else { mem::take(&mut self.items) };
		let (hidden, items) = self.split_files(take);

		self.hidden.extend(hidden);
		if !items.is_empty() {
			self.revision += 1;
			self.items.extend(items);
		}
	}
}
