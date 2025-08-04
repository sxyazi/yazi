use std::ops::Deref;

use yazi_shared::url::Url;

pub struct DirEntry(tokio::fs::DirEntry);

impl Deref for DirEntry {
	type Target = tokio::fs::DirEntry;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<tokio::fs::DirEntry> for DirEntry {
	fn from(value: tokio::fs::DirEntry) -> Self { Self(value) }
}

impl From<DirEntry> for crate::provider::DirEntry {
	fn from(value: DirEntry) -> Self { crate::provider::DirEntry::Local(value) }
}

impl DirEntry {
	#[must_use]
	pub fn url(&self) -> Url { self.0.path().into() }
}

// --- DirEntrySync
pub struct DirEntrySync(std::fs::DirEntry);

impl Deref for DirEntrySync {
	type Target = std::fs::DirEntry;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<std::fs::DirEntry> for DirEntrySync {
	fn from(value: std::fs::DirEntry) -> Self { Self(value) }
}

impl From<DirEntrySync> for crate::provider::DirEntrySync {
	fn from(value: DirEntrySync) -> Self { crate::provider::DirEntrySync::Local(value) }
}

impl DirEntrySync {
	#[must_use]
	pub fn url(&self) -> Url { self.0.path().into() }
}
