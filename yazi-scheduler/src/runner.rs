use std::sync::Arc;

use crate::{TaskIn, TaskOut, file::File, hook::Hook, plugin::Plugin, prework::Prework, process::Process};

#[derive(Clone)]
pub struct Runner {
	pub(super) file:    Arc<File>,
	pub(super) plugin:  Arc<Plugin>,
	pub prework:        Arc<Prework>,
	pub(super) process: Arc<Process>,
	pub(super) hook:    Arc<Hook>,
}

impl Runner {
	pub(super) async fn micro(&self, r#in: TaskIn) -> Result<(), TaskOut> {
		match r#in {
			// File
			TaskIn::FileCopy(r#in) => self.file.copy(r#in).await.map_err(Into::into),
			TaskIn::FileCut(r#in) => self.file.cut(r#in).await.map_err(Into::into),
			TaskIn::FileLink(r#in) => self.file.link(r#in).await.map_err(Into::into),
			TaskIn::FileHardlink(r#in) => self.file.hardlink(r#in).await.map_err(Into::into),
			TaskIn::FileDelete(r#in) => self.file.delete(r#in).await.map_err(Into::into),
			TaskIn::FileTrash(r#in) => self.file.trash(r#in).await.map_err(Into::into),
			TaskIn::FileDownload(r#in) => self.file.download(r#in).await.map_err(Into::into),
			TaskIn::FileUpload(r#in) => self.file.upload(r#in).await.map_err(Into::into),
			// Plugin
			TaskIn::PluginEntry(r#in) => self.plugin.entry(r#in).await.map_err(Into::into),
			// Prework
			TaskIn::PreworkFetch(r#in) => self.prework.fetch(r#in).await.map_err(Into::into),
			TaskIn::PreworkLoad(r#in) => self.prework.load(r#in).await.map_err(Into::into),
			TaskIn::PreworkSize(r#in) => self.prework.size(r#in).await.map_err(Into::into),
			// Process
			TaskIn::ProcessBlock(r#in) => self.process.block(r#in).await.map_err(Into::into),
			TaskIn::ProcessOrphan(r#in) => self.process.orphan(r#in).await.map_err(Into::into),
			TaskIn::ProcessBg(r#in) => self.process.bg(r#in).await.map_err(Into::into),
			// Hook
			TaskIn::HookCopy(r#in) => Ok(self.hook.copy(r#in).await),
			TaskIn::HookCut(r#in) => Ok(self.hook.cut(r#in).await),
			TaskIn::HookDelete(r#in) => Ok(self.hook.delete(r#in).await),
			TaskIn::HookTrash(r#in) => Ok(self.hook.trash(r#in).await),
			TaskIn::HookDownload(r#in) => Ok(self.hook.download(r#in).await),
		}
	}

	pub(super) async fn r#macro(&self, r#in: TaskIn) -> Result<(), TaskOut> {
		match r#in {
			// File
			TaskIn::FileCopy(r#in) => self.file.copy_do(r#in).await.map_err(Into::into),
			TaskIn::FileCut(r#in) => self.file.cut_do(r#in).await.map_err(Into::into),
			TaskIn::FileLink(r#in) => self.file.link_do(r#in).await.map_err(Into::into),
			TaskIn::FileHardlink(r#in) => self.file.hardlink_do(r#in).await.map_err(Into::into),
			TaskIn::FileDelete(r#in) => self.file.delete_do(r#in).await.map_err(Into::into),
			TaskIn::FileTrash(r#in) => self.file.trash_do(r#in).await.map_err(Into::into),
			TaskIn::FileDownload(r#in) => self.file.download_do(r#in).await.map_err(Into::into),
			TaskIn::FileUpload(r#in) => self.file.upload_do(r#in).await.map_err(Into::into),
			// Plugin
			TaskIn::PluginEntry(r#in) => self.plugin.entry_do(r#in).await.map_err(Into::into),
			// Prework
			TaskIn::PreworkFetch(r#in) => self.prework.fetch_do(r#in).await.map_err(Into::into),
			TaskIn::PreworkLoad(r#in) => self.prework.load_do(r#in).await.map_err(Into::into),
			TaskIn::PreworkSize(r#in) => self.prework.size_do(r#in).await.map_err(Into::into),
			// Process
			TaskIn::ProcessBlock(_in) => unreachable!(),
			TaskIn::ProcessOrphan(_in) => unreachable!(),
			TaskIn::ProcessBg(_in) => unreachable!(),
			// Hook
			TaskIn::HookCopy(_in) => unreachable!(),
			TaskIn::HookCut(_in) => unreachable!(),
			TaskIn::HookDelete(_in) => unreachable!(),
			TaskIn::HookTrash(_in) => unreachable!(),
			TaskIn::HookDownload(_in) => unreachable!(),
		}
	}
}
