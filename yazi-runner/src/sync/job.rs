use yazi_shared::{data::Data, scope::{Scope, ScopeLike}};
use yazi_shim::SStr;

use crate::PluginJob;

#[derive(Clone, Debug)]
pub struct SyncJob {
	pub name:  SStr,
	pub block: usize,
	pub args:  Vec<Data>,
	pub scope: Scope,
}

impl PluginJob for SyncJob {
	fn name(&self) -> &SStr { &self.name }
}

impl ScopeLike for SyncJob {
	fn scope(&self) -> &Scope { &self.scope }
}
