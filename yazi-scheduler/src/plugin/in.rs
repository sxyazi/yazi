use std::ops::{Deref, DerefMut};

use yazi_runner::entry::EntryJob;
use yazi_shared::Id;

#[derive(Debug)]
pub(crate) enum PluginIn {
	Entry(PluginInEntry),
}

impl_from_in!(Entry(PluginInEntry));

impl PluginIn {
	pub(crate) fn id(&self) -> Id {
		match self {
			Self::Entry(r#in) => r#in.id,
		}
	}
}

// --- Entry
#[derive(Debug)]
pub(crate) struct PluginInEntry(pub(crate) EntryJob);

impl Deref for PluginInEntry {
	type Target = EntryJob;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for PluginInEntry {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
