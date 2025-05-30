use std::{collections::{HashMap, HashSet}, mem, ops::{Deref, Not}};

use tokio::{fs::{self, DirEntry}, select, sync::mpsc::{self, UnboundedReceiver}};
use yazi_shared::{Id, url::{Url, Urn, UrnBuf}};

use super::{FilesSorter, Filter};
use crate::{FILES_TICKET, File, FilesOp, SortBy, cha::Cha, mounts::PARTITIONS};

#[derive(Default)]
pub struct Files {
	hidden:       Vec<File>,
	items:        Vec<File>,
	ticket:       Id,
	version:      u64,
	pub revision: u64,

	pub sizes: HashMap<UrnBuf, u64>,

	sorter:      FilesSorter,
	filter:      Option<Filter>,
	show_hidden: bool,
}

impl Deref for Files {
	type Target = Vec<File>;

	fn deref(&self) -> &Self::Target { &self.items }
}

impl Files {
	pub fn new(show_hidden: bool) -> Self { Self { show_hidden, ..Default::default() } }

	pub async fn from_dir(dir: &Url) -> std::io::Result<UnboundedReceiver<File>> {
		let mut it = fs::read_dir(dir).await?;
		let (tx, rx) = mpsc::unbounded_channel();

		tokio::spawn(async move {
			while let Ok(Some(item)) = it.next_entry().await {
				select! {
					_ = tx.closed() => break,
					result = item.metadata() => {
						let url = Url::from(item.path());
						_ = tx.send(match result {
							Ok(meta) => File::from_follow(url, meta).await,
							Err(_) => File::from_dummy(url, item.file_type().await.ok())
						});
					}
				}
			}
		});
		Ok(rx)
	}

	pub async fn from_dir_bulk(dir: &Url) -> std::io::Result<Vec<File>> {
		let mut it = fs::read_dir(dir).await?;
		let mut entries = Vec::with_capacity(5000);
		while let Ok(Some(entry)) = it.next_entry().await {
			entries.push(entry);
		}

		let (first, rest) = entries.split_at(entries.len() / 3);
		let (second, third) = rest.split_at(entries.len() / 3);
		async fn go(entries: &[DirEntry]) -> Vec<File> {
			let mut files = Vec::with_capacity(entries.len());
			for entry in entries {
				let url = Url::from(entry.path());
				files.push(match entry.metadata().await {
					Ok(meta) => File::from_follow(url, meta).await,
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

	pub async fn assert_stale(dir: &Url, cha: Cha) -> Option<Cha> {
		use std::io::ErrorKind;
		match Cha::from_url(dir).await {
			Ok(c) if !c.is_dir() => FilesOp::issue_error(dir, ErrorKind::NotADirectory).await,
			Ok(c) if c.hits(cha) && PARTITIONS.read().heuristic(cha) => {}
			Ok(c) => return Some(c),
			Err(e) => FilesOp::issue_error(dir, e.kind()).await,
		}
		None
	}
}

impl Files {
	pub fn update_full(&mut self, files: Vec<File>) {
		self.ticket = FILES_TICKET.next();

		let (hidden, items) = self.split_files(files);
		if !(items.is_empty() && self.items.is_empty()) {
			self.revision += 1;
		}

		(self.hidden, self.items) = (hidden, items);
	}

	pub fn update_part(&mut self, files: Vec<File>, ticket: Id) {
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

	pub fn update_size(&mut self, mut sizes: HashMap<UrnBuf, u64>) {
		if sizes.len() <= 50 {
			sizes.retain(|k, v| self.sizes.get(k) != Some(v));
		}

		if sizes.is_empty() {
			return;
		}

		if self.sorter.by == SortBy::Size {
			self.revision += 1;
		}
		self.sizes.extend(sizes);
	}

	pub fn update_ioerr(&mut self) {
		self.ticket = FILES_TICKET.next();
		self.hidden.clear();
		self.items.clear();
	}

	pub fn update_creating(&mut self, files: Vec<File>) {
		if files.is_empty() {
			return;
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut todo: HashMap<_, _> = $src.into_iter().map(|f| (f.urn_owned(), f)).collect();
				for f in &$dist {
					if todo.remove(f.urn()).is_some() && todo.is_empty() {
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
	pub fn update_deleting(&mut self, urns: HashSet<UrnBuf>) -> Vec<usize> {
		if urns.is_empty() {
			return vec![];
		}

		let (mut hidden, mut items) = if let Some(filter) = &self.filter {
			urns.into_iter().partition(|u| {
				(!self.show_hidden && u.as_urn().is_hidden())
					|| !u.as_urn().name().is_some_and(|s| filter.matches(s))
			})
		} else if self.show_hidden {
			(HashSet::new(), urns)
		} else {
			urns.into_iter().partition(|u| u.as_urn().is_hidden())
		};

		let mut deleted = Vec::with_capacity(items.len());
		if !items.is_empty() {
			let mut i = 0;
			self.items.retain(|f| {
				let b = items.remove(f.urn());
				if b {
					deleted.push(i);
				}
				i += 1;
				!b
			});
		}
		if !hidden.is_empty() {
			self.hidden.retain(|f| !hidden.remove(f.urn()));
		}

		self.revision += deleted.is_empty().not() as u64;
		deleted
	}

	#[cfg(windows)]
	pub fn update_deleting(&mut self, mut urns: HashSet<UrnBuf>) -> Vec<usize> {
		let mut deleted = Vec::with_capacity(urns.len());
		if !urns.is_empty() {
			let mut i = 0;
			self.items.retain(|f| {
				let b = urns.remove(f.urn());
				if b {
					deleted.push(i)
				}
				i += 1;
				!b
			});
		}
		if !urns.is_empty() {
			self.hidden.retain(|f| !urns.remove(f.urn()));
		}

		self.revision += deleted.is_empty().not() as u64;
		deleted
	}

	pub fn update_updating(
		&mut self,
		files: HashMap<UrnBuf, File>,
	) -> (HashMap<UrnBuf, File>, HashMap<UrnBuf, File>) {
		if files.is_empty() {
			return Default::default();
		}

		macro_rules! go {
			($dist:expr, $src:expr, $inc:literal) => {
				let mut b = true;
				for i in 0..$dist.len() {
					if let Some(f) = $src.remove($dist[i].urn()) {
						b = b && $dist[i].cha.hits(f.cha);
						b = b && $dist[i].urn() == f.urn();

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
			files
				.into_iter()
				.partition(|(_, f)| (f.is_hidden() && !self.show_hidden) || !filter.matches(f.name()))
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

	pub fn update_upserting(&mut self, files: HashMap<UrnBuf, File>) {
		if files.is_empty() {
			return;
		}

		self.update_deleting(
			files.iter().filter(|&(u, f)| u != f.urn()).map(|(_, f)| f.urn_owned()).collect(),
		);

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
			files
				.into_iter()
				.partition(|f| (f.is_hidden() && !self.show_hidden) || !filter.matches(f.name()))
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
	pub fn position(&self, urn: &Urn) -> Option<usize> { self.iter().position(|f| urn == f.urn()) }

	// --- Ticket
	#[inline]
	pub fn ticket(&self) -> Id { self.ticket }

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
