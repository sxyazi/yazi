use yazi_shared::Id;

use crate::{file::{FileInCopy, FileInCut, FileInDelete, FileInDownload, FileInHardlink, FileInLink, FileInTrash, FileInUpload}, impl_from_in, plugin::PluginInEntry, prework::{PreworkInFetch, PreworkInLoad, PreworkInSize}};

#[derive(Debug)]
pub(crate) enum TaskIn {
	// File
	FileCopy(FileInCopy),
	FileCut(FileInCut),
	FileLink(FileInLink),
	FileHardlink(FileInHardlink),
	FileDelete(FileInDelete),
	FileTrash(FileInTrash),
	FileDownload(FileInDownload),
	FileUpload(FileInUpload),
	// Plugin
	PluginEntry(PluginInEntry),
	// Prework
	PreworkFetch(PreworkInFetch),
	PreworkLoad(PreworkInLoad),
	PreworkSize(PreworkInSize),
}

impl_from_in! {
	// File
	FileCopy(FileInCopy), FileCut(FileInCut), FileLink(FileInLink), FileHardlink(FileInHardlink), FileDelete(FileInDelete), FileTrash(FileInTrash), FileDownload(FileInDownload), FileUpload(FileInUpload),
	// Plugin
	PluginEntry(PluginInEntry),
	// Prework
	PreworkFetch(PreworkInFetch), PreworkLoad(PreworkInLoad), PreworkSize(PreworkInSize),
}

impl TaskIn {
	pub fn id(&self) -> Id {
		match self {
			// File
			Self::FileCopy(r#in) => r#in.id,
			Self::FileCut(r#in) => r#in.id,
			Self::FileLink(r#in) => r#in.id,
			Self::FileHardlink(r#in) => r#in.id,
			Self::FileDelete(r#in) => r#in.id,
			Self::FileTrash(r#in) => r#in.id,
			Self::FileDownload(r#in) => r#in.id,
			Self::FileUpload(r#in) => r#in.id,
			// Plugin
			Self::PluginEntry(r#in) => r#in.id,
			// Prework
			Self::PreworkFetch(r#in) => r#in.id,
			Self::PreworkLoad(r#in) => r#in.id,
			Self::PreworkSize(r#in) => r#in.id,
		}
	}
}
