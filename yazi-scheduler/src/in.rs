use yazi_shared::Id;

use crate::{file::{FileInCopy, FileInCut, FileInDelete, FileInDownload, FileInHardlink, FileInLink, FileInTrash, FileInUpload}, hook::{HookInOutBg, HookInOutBlock, HookInOutCut, HookInOutDelete, HookInOutDownload, HookInOutFetch, HookInOutOrphan, HookInOutTrash}, impl_from_in, plugin::PluginInEntry, prework::{PreworkInFetch, PreworkInLoad, PreworkInSize}, process::{ProcessInBg, ProcessInBlock, ProcessInOrphan}};

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
	// Process
	ProcessBlock(ProcessInBlock),
	ProcessOrphan(ProcessInOrphan),
	ProcessBg(ProcessInBg),
	// Hook
	HookCut(HookInOutCut),
	HookDelete(HookInOutDelete),
	HookTrash(HookInOutTrash),
	HookDownload(HookInOutDownload),
	HookBlock(HookInOutBlock),
	HookOrphan(HookInOutOrphan),
	HookBg(HookInOutBg),
	HookFetch(HookInOutFetch),
}

impl_from_in! {
	// File
	FileCopy(FileInCopy), FileCut(FileInCut), FileLink(FileInLink), FileHardlink(FileInHardlink), FileDelete(FileInDelete), FileTrash(FileInTrash), FileDownload(FileInDownload), FileUpload(FileInUpload),
	// Plugin
	PluginEntry(PluginInEntry),
	// Prework
	PreworkFetch(PreworkInFetch), PreworkLoad(PreworkInLoad), PreworkSize(PreworkInSize),
	// Process
	ProcessBlock(ProcessInBlock), ProcessOrphan(ProcessInOrphan), ProcessBg(ProcessInBg),
	// Hook
	HookCut(HookInOutCut), HookDelete(HookInOutDelete), HookTrash(HookInOutTrash), HookDownload(HookInOutDownload), HookBlock(HookInOutBlock), HookOrphan(HookInOutOrphan), HookBg(HookInOutBg), HookFetch(HookInOutFetch),
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
			// Process
			Self::ProcessBlock(r#in) => r#in.id,
			Self::ProcessOrphan(r#in) => r#in.id,
			Self::ProcessBg(r#in) => r#in.id,
			// Hook
			Self::HookCut(r#in) => r#in.id,
			Self::HookDelete(r#in) => r#in.id,
			Self::HookTrash(r#in) => r#in.id,
			Self::HookDownload(r#in) => r#in.id,
			Self::HookBlock(r#in) => r#in.id,
			Self::HookOrphan(r#in) => r#in.id,
			Self::HookBg(r#in) => r#in.id,
			Self::HookFetch(r#in) => r#in.id,
		}
	}
}
