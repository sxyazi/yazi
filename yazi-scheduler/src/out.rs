use crate::{file::FileOut, plugin::PluginOut, prework::PreworkOut};

#[derive(Debug)]
pub enum TaskOut {
	File(FileOut),
	Plugin(PluginOut),
	Prework(PreworkOut),
}
