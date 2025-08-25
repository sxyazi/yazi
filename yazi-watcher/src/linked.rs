use std::{iter, ops::{Deref, DerefMut}, path::{Path, PathBuf}};

use hashbrown::{HashMap, HashSet};
use parking_lot::RwLock;
use yazi_shared::url::Url;

use crate::Watched;

#[derive(Default)]
pub struct Linked(HashMap<PathBuf, PathBuf> /* from ==> to */);

impl Deref for Linked {
	type Target = HashMap<PathBuf, PathBuf>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Linked {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Linked {
	pub fn from_dir<'a, 'b, T>(&'a self, url: T) -> Box<dyn Iterator<Item = &'a Path> + 'b>
	where
		'a: 'b,
		T: Into<Url<'b>>,
	{
		let url: Url = url.into();
		let Some(path) = url.as_path() else {
			return Box::new(iter::empty());
		};

		if let Some(to) = self.get(path) {
			Box::new(self.iter().filter(move |(k, v)| *v == to && *k != path).map(|(k, _)| k.as_path()))
		} else {
			Box::new(self.iter().filter(move |(_, v)| *v == path).map(|(k, _)| k.as_path()))
		}
	}

	pub fn from_file(&self, url: Url) -> Vec<PathBuf> {
		let Some(path) = url.as_path() else { return vec![] };
		if let Some((parent, name)) = path.parent().zip(path.file_name()) {
			self.from_dir(Url::regular(parent)).map(|p| p.join(name)).collect()
		} else {
			vec![]
		}
	}

	pub(super) async fn sync(linked: &'static RwLock<Self>, watched: &'static RwLock<Watched>) {
		tokio::task::spawn_blocking(move || {
			let mut new: HashSet<_> = watched.read().paths().map(ToOwned::to_owned).collect();
			let mut linked = linked.write();

			linked.retain(|k, _| new.remove(k));
			for from in new {
				linked.insert(from, PathBuf::new());
			}

			for (from, to) in linked.iter_mut() {
				match std::fs::canonicalize(from) {
					Ok(c) if c != *from && watched.read().contains(Url::regular(from)) => *to = c,
					Ok(_) => *to = PathBuf::new(),
					Err(e) if e.kind() == std::io::ErrorKind::NotFound => *to = PathBuf::new(),
					Err(_) => {}
				}
			}
			linked.retain(|_, v| !v.as_os_str().is_empty());
		})
		.await
		.ok();
	}
}
