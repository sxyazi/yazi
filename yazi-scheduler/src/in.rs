use yazi_shared::Id;

use crate::{file::{FileInDelete, FileInHardlink, FileInLink, FileInPaste, FileInTrash}, impl_from_in, plugin::PluginInEntry, prework::{PreworkInFetch, PreworkInLoad, PreworkInSize}};

#[derive(Debug)]
pub(crate) enum TaskIn {
	// File
	FilePaste(FileInPaste),
	FileLink(FileInLink),
	FileHardlink(FileInHardlink),
	FileDelete(FileInDelete),
	FileTrash(FileInTrash),

	// Plugin
	PluginEntry(PluginInEntry),

	// Prework
	PreworkFetch(PreworkInFetch),
	PreworkLoad(PreworkInLoad),
	PreworkSize(PreworkInSize),
}

impl_from_in! {
	// File
	FilePaste(FileInPaste), FileLink(FileInLink), FileHardlink(FileInHardlink), FileDelete(FileInDelete), FileTrash(FileInTrash),
	// Plugin
	PluginEntry(PluginInEntry),
	// Prework
	PreworkFetch(PreworkInFetch), PreworkLoad(PreworkInLoad), PreworkSize(PreworkInSize),
}

impl TaskIn {
	pub fn id(&self) -> Id {
		match self {
			// File
			Self::FilePaste(r#in) => r#in.id,
			Self::FileLink(r#in) => r#in.id,
			Self::FileHardlink(r#in) => r#in.id,
			Self::FileDelete(r#in) => r#in.id,
			Self::FileTrash(r#in) => r#in.id,

			// Plugin
			Self::PluginEntry(r#in) => r#in.id,

			// Prework
			Self::PreworkFetch(r#in) => r#in.id,
			Self::PreworkLoad(r#in) => r#in.id,
			Self::PreworkSize(r#in) => r#in.id,
		}
	}
}
