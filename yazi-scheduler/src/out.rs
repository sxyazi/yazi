use crate::{Task, custom::CustomOut, fetch::FetchOutFetch, file::{FileOutCopy, FileOutCopyDo, FileOutDelete, FileOutDeleteDo, FileOutDownload, FileOutDownloadDo, FileOutHardlink, FileOutHardlinkDo, FileOutLink, FileOutMove, FileOutMoveDo, FileOutTrash, FileOutUpload, FileOutUploadDo}, hook::{HookInOutCopy, HookInOutHardlink, HookInOutLink, HookInOutMove}, impl_from_out, plugin::PluginOutEntry, preload::PreloadOut, process::{ProcessOutBg, ProcessOutBlock, ProcessOutOrphan}, size::SizeOut};

#[derive(Debug)]
pub(super) enum TaskOut {
	// File
	FileCopy(FileOutCopy),
	FileCopyDo(FileOutCopyDo),
	FileMove(FileOutMove),
	FileMoveDo(FileOutMoveDo),
	FileLink(FileOutLink),
	FileHardlink(FileOutHardlink),
	FileHardlinkDo(FileOutHardlinkDo),
	FileDelete(FileOutDelete),
	FileDeleteDo(FileOutDeleteDo),
	FileTrash(FileOutTrash),
	FileDownload(FileOutDownload),
	FileDownloadDo(FileOutDownloadDo),
	FileUpload(FileOutUpload),
	FileUploadDo(FileOutUploadDo),
	// Plugin
	PluginEntry(PluginOutEntry),
	// Fetch
	Fetch(FetchOutFetch),
	// Preload
	Preload(PreloadOut),
	// Size
	Size(SizeOut),
	// Process
	ProcessBlock(ProcessOutBlock),
	ProcessOrphan(ProcessOutOrphan),
	ProcessBg(ProcessOutBg),
	// Custom
	Custom(CustomOut),
	// Hook
	HookCopy(HookInOutCopy),
	HookMove(HookInOutMove),
	HookLink(HookInOutLink),
	HookHardlink(HookInOutHardlink),
}

impl_from_out! {
	// File
	FileCopy(FileOutCopy), FileCopyDo(FileOutCopyDo), FileMove(FileOutMove), FileMoveDo(FileOutMoveDo), FileLink(FileOutLink), FileHardlink(FileOutHardlink), FileHardlinkDo(FileOutHardlinkDo), FileDelete(FileOutDelete), FileDeleteDo(FileOutDeleteDo), FileTrash(FileOutTrash), FileDownload(FileOutDownload), FileDownloadDo(FileOutDownloadDo), FileUpload(FileOutUpload), FileUploadDo(FileOutUploadDo),
	// Plugin
	PluginEntry(PluginOutEntry),
	// Fetch
	Fetch(FetchOutFetch),
	// Preload
	Preload(PreloadOut),
	// Size
	Size(SizeOut),
	// Process
	ProcessBlock(ProcessOutBlock), ProcessOrphan(ProcessOutOrphan), ProcessBg(ProcessOutBg),
	// Custom
	Custom(CustomOut),
	// Hook
	HookCopy(HookInOutCopy), HookMove(HookInOutMove), HookLink(HookInOutLink), HookHardlink(HookInOutHardlink),
}

impl TaskOut {
	pub(crate) fn reduce(self, task: &mut Task) {
		match self {
			// File
			Self::FileCopy(out) => out.reduce(task),
			Self::FileCopyDo(out) => out.reduce(task),
			Self::FileMove(out) => out.reduce(task),
			Self::FileMoveDo(out) => out.reduce(task),
			Self::FileLink(out) => out.reduce(task),
			Self::FileHardlink(out) => out.reduce(task),
			Self::FileHardlinkDo(out) => out.reduce(task),
			Self::FileDelete(out) => out.reduce(task),
			Self::FileDeleteDo(out) => out.reduce(task),
			Self::FileTrash(out) => out.reduce(task),
			Self::FileDownload(out) => out.reduce(task),
			Self::FileDownloadDo(out) => out.reduce(task),
			Self::FileUpload(out) => out.reduce(task),
			Self::FileUploadDo(out) => out.reduce(task),
			// Plugin
			Self::PluginEntry(out) => out.reduce(task),
			// Prework
			Self::Fetch(out) => out.reduce(task),
			Self::Preload(out) => out.reduce(task),
			Self::Size(out) => out.reduce(task),
			// Process
			Self::ProcessBlock(out) => out.reduce(task),
			Self::ProcessOrphan(out) => out.reduce(task),
			Self::ProcessBg(out) => out.reduce(task),
			// Custom
			Self::Custom(out) => out.reduce(task),
			// Hook
			Self::HookCopy(out) => out.reduce(task),
			Self::HookMove(out) => out.reduce(task),
			Self::HookLink(out) => out.reduce(task),
			Self::HookHardlink(out) => out.reduce(task),
		}
	}
}
