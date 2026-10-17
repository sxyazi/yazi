use yazi_binding::Scope;
use yazi_shared::{data::Data, id::Id};
use yazi_shim::SStr;

use crate::PluginJob;

#[derive(Clone, Debug)]
pub struct SyncJob {
	pub tab:   Id,
	pub name:  SStr,
	pub block: usize,
	pub args:  Vec<Data>,
	pub scope: Scope,
}

impl PluginJob for SyncJob {
	fn tab(&self) -> Id { self.tab }

	fn name(&self) -> &SStr { &self.name }

	fn scope(&self) -> &Scope { &self.scope }
}
