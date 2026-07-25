use std::vec;

use mlua::{FromLua, Lua, Value};

use super::{TrashEntry, TrashId};

pub(crate) struct TrashEntries(Vec<TrashEntry>);

impl TrashEntries {
	fn new(mut entries: Vec<TrashEntry>) -> Self {
		entries.sort_unstable_by_key(|entry| entry.rel().components().count());

		let mut seen = Vec::<TrashId>::with_capacity(entries.len());
		entries.retain(|entry| {
			if seen.iter().any(|id| id.top() == entry.top() && entry.rel().starts_with(id.rel())) {
				false
			} else {
				seen.push(entry.id.clone());
				true
			}
		});
		Self(entries)
	}
}

impl FromLua for TrashEntries {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let entries = Vec::<TrashEntry>::from_lua(value, lua)?;
		Ok(Self::new(entries))
	}
}

impl IntoIterator for TrashEntries {
	type IntoIter = vec::IntoIter<TrashEntry>;
	type Item = TrashEntry;

	fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
