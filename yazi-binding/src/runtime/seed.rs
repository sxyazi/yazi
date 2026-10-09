use compact_str::CompactString;
use yazi_shared::id::Id;

use crate::Scope;

#[derive(Clone, Debug, Default)]
pub struct RuntimeSeed {
	pub tab:   Id,
	pub name:  CompactString,
	pub scope: Scope,
}

impl RuntimeSeed {
	pub fn new(tab: Id, name: impl Into<CompactString>, scope: Scope) -> Self {
		Self { tab, name: name.into(), scope }
	}

	pub fn fork(&mut self) -> Scope {
		self.scope = self.scope.child();
		self.scope.clone()
	}
}
