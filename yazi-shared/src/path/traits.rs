use std::hash::BuildHasher;

use hashbrown::{HashMap, hash_map::EntryRef};

use crate::path::{PathBufDyn, PathDyn};

// --- PathMapExt
pub trait PathMapExt<V> {
	fn get_or_insert_default(&mut self, path: PathDyn<'_>) -> &mut V
	where
		V: Default;
}

impl<V, S> PathMapExt<V> for HashMap<PathBufDyn, V, S>
where
	S: BuildHasher,
{
	fn get_or_insert_default(&mut self, path: PathDyn<'_>) -> &mut V
	where
		V: Default,
	{
		match self.entry_ref(&path) {
			EntryRef::Occupied(oe) => oe.into_mut(),
			EntryRef::Vacant(ve) => ve.insert_with_key(path.to_owned(), Default::default()),
		}
	}
}
