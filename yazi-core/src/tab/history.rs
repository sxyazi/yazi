use std::collections::VecDeque;

use hashbrown::{HashMap, HashSet};
use yazi_shared::{path::{PathBufDyn, PathMapExt}, url::{AsUrl, UrlBuf, UrlLike, UrlMapExt}};

use super::Folder;

#[derive(Default)]
pub struct History {
	entries: HashMap<UrlBuf, Folder>,
	queue:   VecDeque<UrlBuf>,
	indices: HashMap<UrlBuf, HashMap<PathBufDyn, Vec<UrlBuf>>>,
}

impl History {
	pub fn get(&self, url: &UrlBuf) -> Option<&Folder> { self.entries.get(url) }

	pub(crate) fn get_mut(&mut self, url: &UrlBuf) -> Option<&mut Folder> {
		self.entries.get_mut(url)
	}

	pub fn ensure(&mut self, url: &UrlBuf) -> (&mut Folder, Option<Folder>) {
		let evicted =
			if self.entries.contains_key(url) { None } else { self.insert(Folder::from(url)) };
		(self.entries.get_mut(url).unwrap(), evicted)
	}

	pub fn insert(&mut self, folder: Folder) -> Option<Folder> {
		self.remove(&folder.url);

		if let Some((trail, key)) = folder.url.pair() {
			self.indices.get_or_insert_default(trail).get_or_insert_default(key).push(folder.url.clone());
		}

		self.queue.push_back(folder.url.clone());
		self.entries.insert(folder.url.clone(), folder);

		(self.queue.len() > 100).then(|| {
			let url = self.queue.pop_front().unwrap();
			self.remove(&url).unwrap()
		})
	}

	pub fn remove_or(&mut self, url: impl AsUrl) -> Folder {
		let url = url.as_url();
		self.remove(url).unwrap_or_else(|| Folder::from(url))
	}

	pub(crate) fn for_each_mut<F>(&mut self, trail: &UrlBuf, keys: &HashSet<PathBufDyn>, mut f: F)
	where
		F: FnMut(&mut Folder),
	{
		let urls = self
			.indices
			.get(trail)
			.into_iter()
			.flat_map(|index| keys.iter().filter_map(|key| index.get(key)).flatten());

		for url in urls {
			if let Some(folder) = self.entries.get_mut(url) {
				f(folder);
			}
		}
	}

	fn remove(&mut self, url: impl AsUrl) -> Option<Folder> {
		let (url, folder) = self.entries.remove_entry(&url.as_url())?;
		self.queue.retain(|other| other != &url);
		self.unindex(&url);
		Some(folder)
	}

	fn unindex(&mut self, url: &UrlBuf) {
		let Some((trail, key)) = url.pair() else { return };
		let Some(index) = self.indices.get_mut(&trail) else { return };

		let remove = index.get_mut(&key).is_some_and(|urls| {
			urls.retain(|other| other != url);
			urls.is_empty()
		});

		if remove {
			index.remove(&key);
		}

		if index.is_empty() {
			self.indices.remove(&trail);
		}
	}
}
