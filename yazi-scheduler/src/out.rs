use crate::{Task, file::{FileOutCopy, FileOutCopyDo, FileOutCut, FileOutCutDo, FileOutDelete, FileOutDeleteDo, FileOutDownload, FileOutDownloadDo, FileOutHardlink, FileOutHardlinkDo, FileOutLink, FileOutTrash, FileOutUpload, FileOutUploadDo}, hook::{HookInOutCopy, HookInOutCut, HookInOutDelete, HookInOutDownload, HookInOutTrash}, impl_from_out, plugin::PluginOutEntry, prework::{PreworkOutFetch, PreworkOutLoad, PreworkOutSize}, process::{ProcessOutBg, ProcessOutBlock, ProcessOutOrphan}};

#[derive(Debug)]
pub(super) enum TaskOut {
	// File
	FileCopy(FileOutCopy),
	FileCopyDo(FileOutCopyDo),
	FileCut(FileOutCut),
	FileCutDo(FileOutCutDo),
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
	// Prework
	PreworkFetch(PreworkOutFetch),
	PreworkLoad(PreworkOutLoad),
	PreworkSize(PreworkOutSize),
	// Process
	ProcessBlock(ProcessOutBlock),
	ProcessOrphan(ProcessOutOrphan),
	ProcessBg(ProcessOutBg),
	// Hook
	HookCopy(HookInOutCopy),
	HookCut(HookInOutCut),
	HookDelete(HookInOutDelete),
	HookTrash(HookInOutTrash),
	HookDownload(HookInOutDownload),
}

impl_from_out! {
	// File
	FileCopy(FileOutCopy), FileCopyDo(FileOutCopyDo), FileCut(FileOutCut), FileCutDo(FileOutCutDo), FileLink(FileOutLink), FileHardlink(FileOutHardlink), FileHardlinkDo(FileOutHardlinkDo), FileDelete(FileOutDelete), FileDeleteDo(FileOutDeleteDo), FileTrash(FileOutTrash), FileDownload(FileOutDownload), FileDownloadDo(FileOutDownloadDo), FileUpload(FileOutUpload), FileUploadDo(FileOutUploadDo),
	// Plugin
	PluginEntry(PluginOutEntry),
	// Prework
	PreworkFetch(PreworkOutFetch), PreworkLoad(PreworkOutLoad), PreworkSize(PreworkOutSize),
	// Process
	ProcessBlock(ProcessOutBlock), ProcessOrphan(ProcessOutOrphan), ProcessBg(ProcessOutBg),
	// Hook
	HookCopy(HookInOutCopy), HookCut(HookInOutCut), HookDelete(HookInOutDelete), HookTrash(HookInOutTrash), HookDownload(HookInOutDownload),
}

impl TaskOut {
	pub(crate) fn reduce(self, task: &mut Task) {
		match self {
			// File
			Self::FileCopy(out) => out.reduce(task),
			Self::FileCopyDo(out) => out.reduce(task),
			Self::FileCut(out) => out.reduce(task),
			Self::FileCutDo(out) => out.reduce(task),
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
			Self::PreworkFetch(out) => out.reduce(task),
			Self::PreworkLoad(out) => out.reduce(task),
			Self::PreworkSize(out) => out.reduce(task),
			// Process
			Self::ProcessBlock(out) => out.reduce(task),
			Self::ProcessOrphan(out) => out.reduce(task),
			Self::ProcessBg(out) => out.reduce(task),
			// Hook
			Self::HookCopy(out) => out.reduce(task),
			Self::HookCut(out) => out.reduce(task),
			Self::HookDelete(out) => out.reduce(task),
			Self::HookTrash(out) => out.reduce(task),
			Self::HookDownload(out) => out.reduce(task),
		}
	}
}
