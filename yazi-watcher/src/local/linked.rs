use std::{iter, ops::{Deref, DerefMut}, path::{Path, PathBuf}};

use hashbrown::HashMap;
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
	pub fn from_dir<'a, 'b, U>(&'a self, url: U) -> Box<dyn Iterator<Item = &'a Path> + 'b>
	where
		'a: 'b,
		U: Into<Url<'b>>,
	{
		let url: Url = url.into();
		let Some(path) = url.as_local() else {
			return Box::new(iter::empty());
		};

		if let Some(to) = self.get(path) {
			Box::new(self.iter().filter(move |(k, v)| *v == to && *k != path).map(|(k, _)| k.as_path()))
		} else {
			Box::new(self.iter().filter(move |(_, v)| *v == path).map(|(k, _)| k.as_path()))
		}
	}

	pub fn from_file(&self, url: Url) -> Vec<PathBuf> {
		let Some(path) = url.as_local() else { return vec![] };
		if let Some((parent, name)) = path.parent().zip(path.file_name()) {
			self.from_dir(parent).map(|p| p.join(name)).collect()
		} else {
			vec![]
		}
	}

	pub(crate) async fn sync(linked: &'static RwLock<Self>, watched: &'static RwLock<Watched>) {
		tokio::task::spawn_blocking(move || {
			let watched = watched.read();

			// Remove entries that are no longer watched
			linked.write().retain(|from, _| watched.contains(from));

			// Update existing entries and remove broken links
			for from in watched.paths() {
				match std::fs::canonicalize(from) {
					Ok(to) if to != *from => _ = linked.write().entry_ref(from).insert(to),
					Ok(_) => _ = linked.write().remove(from),
					Err(e) if e.kind() == std::io::ErrorKind::NotFound => _ = linked.write().remove(from),
					Err(e) => tracing::error!("Failed to canonicalize watched path {from:?}: {e:?}"),
				}
			}
		})
		.await
		.ok();
	}
}
