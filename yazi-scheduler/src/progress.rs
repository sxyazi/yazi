use serde::Serialize;
use yazi_parser::app::TaskSummary;

use crate::{file::{FileProgDelete, FileProgHardlink, FileProgLink, FileProgPaste, FileProgTrash}, impl_from_prog, plugin::PluginProgEntry, prework::{PreworkProgFetch, PreworkProgLoad, PreworkProgSize}, process::{ProcessProgBg, ProcessProgBlock, ProcessProgOrphan}};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub enum TaskProg {
	// File
	FilePaste(FileProgPaste),
	FileLink(FileProgLink),
	FileHardlink(FileProgHardlink),
	FileDelete(FileProgDelete),
	FileTrash(FileProgTrash),
	// Plugin
	PluginEntry(PluginProgEntry),
	// Prework
	PreworkFetch(PreworkProgFetch),
	PreworkLoad(PreworkProgLoad),
	PreworkSize(PreworkProgSize),
	// Process
	ProcessBlock(ProcessProgBlock),
	ProcessOrphan(ProcessProgOrphan),
	ProcessBg(ProcessProgBg),
}

impl_from_prog! {
	// File
	FilePaste(FileProgPaste), FileLink(FileProgLink), FileHardlink(FileProgHardlink), FileDelete(FileProgDelete), FileTrash(FileProgTrash),
	// Plugin
	PluginEntry(PluginProgEntry),
	// Prework
	PreworkFetch(PreworkProgFetch), PreworkLoad(PreworkProgLoad), PreworkSize(PreworkProgSize),
	// Process
	ProcessBlock(ProcessProgBlock), ProcessOrphan(ProcessProgOrphan), ProcessBg(ProcessProgBg),
}

impl From<TaskProg> for TaskSummary {
	fn from(value: TaskProg) -> Self {
		match value {
			// File
			TaskProg::FilePaste(p) => p.into(),
			TaskProg::FileLink(p) => p.into(),
			TaskProg::FileHardlink(p) => p.into(),
			TaskProg::FileDelete(p) => p.into(),
			TaskProg::FileTrash(p) => p.into(),
			// Plugin
			TaskProg::PluginEntry(p) => p.into(),
			// Prework
			TaskProg::PreworkFetch(p) => p.into(),
			TaskProg::PreworkLoad(p) => p.into(),
			TaskProg::PreworkSize(p) => p.into(),
			// Process
			TaskProg::ProcessBlock(p) => p.into(),
			TaskProg::ProcessOrphan(p) => p.into(),
			TaskProg::ProcessBg(p) => p.into(),
		}
	}
}

impl TaskProg {
	pub fn running(self) -> bool {
		match self {
			// File
			Self::FilePaste(p) => p.running(),
			Self::FileLink(p) => p.running(),
			Self::FileHardlink(p) => p.running(),
			Self::FileDelete(p) => p.running(),
			Self::FileTrash(p) => p.running(),
			// Plugin
			Self::PluginEntry(p) => p.running(),
			// Prework
			Self::PreworkFetch(p) => p.running(),
			Self::PreworkLoad(p) => p.running(),
			Self::PreworkSize(p) => p.running(),
			// Process
			Self::ProcessBlock(p) => p.running(),
			Self::ProcessOrphan(p) => p.running(),
			Self::ProcessBg(p) => p.running(),
		}
	}

	pub fn success(self) -> bool {
		match self {
			// File
			Self::FilePaste(p) => p.success(),
			Self::FileLink(p) => p.success(),
			Self::FileHardlink(p) => p.success(),
			Self::FileDelete(p) => p.success(),
			Self::FileTrash(p) => p.success(),
			// Plugin
			Self::PluginEntry(p) => p.success(),
			// Prework
			Self::PreworkFetch(p) => p.success(),
			Self::PreworkLoad(p) => p.success(),
			Self::PreworkSize(p) => p.success(),
			// Process
			Self::ProcessBlock(p) => p.success(),
			Self::ProcessOrphan(p) => p.success(),
			Self::ProcessBg(p) => p.success(),
		}
	}

	pub fn cleaned(self) -> bool {
		match self {
			// File
			Self::FilePaste(p) => p.cleaned(),
			Self::FileLink(p) => p.cleaned(),
			Self::FileHardlink(p) => p.cleaned(),
			Self::FileDelete(p) => p.cleaned(),
			Self::FileTrash(p) => p.cleaned(),
			// Plugin
			Self::PluginEntry(p) => p.cleaned(),
			// Prework
			Self::PreworkFetch(p) => p.cleaned(),
			Self::PreworkLoad(p) => p.cleaned(),
			Self::PreworkSize(p) => p.cleaned(),
			// Process
			Self::ProcessBlock(p) => p.cleaned(),
			Self::ProcessOrphan(p) => p.cleaned(),
			Self::ProcessBg(p) => p.cleaned(),
		}
	}

	pub fn percent(self) -> Option<f32> {
		match self {
			// File
			Self::FilePaste(p) => p.percent(),
			Self::FileLink(p) => p.percent(),
			Self::FileHardlink(p) => p.percent(),
			Self::FileDelete(p) => p.percent(),
			Self::FileTrash(p) => p.percent(),
			// Plugin
			Self::PluginEntry(p) => p.percent(),
			// Prework
			Self::PreworkFetch(p) => p.percent(),
			Self::PreworkLoad(p) => p.percent(),
			Self::PreworkSize(p) => p.percent(),
			// Process
			Self::ProcessBlock(p) => p.percent(),
			Self::ProcessOrphan(p) => p.percent(),
			Self::ProcessBg(p) => p.percent(),
		}
	}

	pub(crate) fn is_user(self) -> bool {
		match self {
			// File
			Self::FilePaste(_) => true,
			Self::FileLink(_) => true,
			Self::FileHardlink(_) => true,
			Self::FileDelete(_) => true,
			Self::FileTrash(_) => true,
			// Plugin
			Self::PluginEntry(_) => true,
			// Prework
			Self::PreworkFetch(_) => false,
			Self::PreworkLoad(_) => false,
			Self::PreworkSize(_) => false,
			// Process
			Self::ProcessBlock(_) => true,
			Self::ProcessOrphan(_) => true,
			Self::ProcessBg(_) => true,
		}
	}
}
