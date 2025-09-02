use crate::{Task, file::{FileOutDelete, FileOutDeleteDo, FileOutHardlink, FileOutHardlinkDo, FileOutLink, FileOutPaste, FileOutPasteDo, FileOutTrash}, impl_from_out, plugin::PluginOutEntry, prework::{PreworkOutFetch, PreworkOutLoad, PreworkOutSize}, process::{ProcessOutBg, ProcessOutBlock, ProcessOutOrphan}};

#[derive(Debug)]
pub(super) enum TaskOut {
	// File
	FilePaste(FileOutPaste),
	FilePasteDo(FileOutPasteDo),
	FileLink(FileOutLink),
	FileHardlink(FileOutHardlink),
	FileHardlinkDo(FileOutHardlinkDo),
	FileDelete(FileOutDelete),
	FileDeleteDo(FileOutDeleteDo),
	FileTrash(FileOutTrash),

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
}

impl_from_out! {
	// File
	FilePaste(FileOutPaste), FilePasteDo(FileOutPasteDo), FileLink(FileOutLink), FileHardlink(FileOutHardlink), FileHardlinkDo(FileOutHardlinkDo), FileDelete(FileOutDelete), FileDeleteDo(FileOutDeleteDo), FileTrash(FileOutTrash),
	// Plugin
	PluginEntry(PluginOutEntry),
	// Prework
	PreworkFetch(PreworkOutFetch), PreworkLoad(PreworkOutLoad), PreworkSize(PreworkOutSize),
	// Process
	ProcessBlock(ProcessOutBlock), ProcessOrphan(ProcessOutOrphan), ProcessBg(ProcessOutBg),
}

impl TaskOut {
	pub(crate) fn reduce(self, task: &mut Task) {
		match self {
			// File
			Self::FilePaste(out) => out.reduce(task),
			Self::FilePasteDo(out) => out.reduce(task),
			Self::FileLink(out) => out.reduce(task),
			Self::FileHardlink(out) => out.reduce(task),
			Self::FileHardlinkDo(out) => out.reduce(task),
			Self::FileDelete(out) => out.reduce(task),
			Self::FileDeleteDo(out) => out.reduce(task),
			Self::FileTrash(out) => out.reduce(task),
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
		}
	}
}
