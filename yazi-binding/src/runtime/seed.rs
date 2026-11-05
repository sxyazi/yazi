use yazi_shared::scope::{Scope, ScopeLike};
use yazi_shim::CompactString;

#[derive(Clone, Debug, Default)]
pub struct RuntimeSeed {
	pub name:  CompactString,
	pub scope: Scope,
}

impl RuntimeSeed {
	pub fn new(name: impl Into<CompactString>, scope: impl Into<Scope>) -> Self {
		Self { name: name.into(), scope: scope.into() }
	}
}

impl ScopeLike for RuntimeSeed {
	fn scope(&self) -> &Scope { &self.scope }
}
