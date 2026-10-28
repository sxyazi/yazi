use std::borrow::Cow;

use yazi_config::plugin::FetcherArc;
use yazi_fs::file::Files;
use yazi_runner::fetcher::FetchJob;
use yazi_shared::{id::Id, scope::{Scope, ScopeLike}};

use crate::{TaskIn, custom::CustomIn, fetch::FetchProg};

#[derive(Debug)]
pub(crate) enum FetchIn {
	Fetch(FetchInFetch),
	Custom(CustomIn),
}

impl TaskIn for FetchIn {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Fetch(r#in) => r#in.id(),
			Self::Custom(r#in) => r#in.id(),
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Fetch(r#in) => _ = r#in.set_id(id),
			Self::Custom(r#in) => _ = r#in.set_id(id),
		}
		self
	}

	fn scope(&self) -> &Scope {
		match self {
			Self::Fetch(r#in) => r#in.scope(),
			Self::Custom(r#in) => r#in.scope(),
		}
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Fetch(r#in) => r#in.title(),
			Self::Custom(r#in) => r#in.title(),
		}
	}
}

impl_from_in!(Fetch(FetchInFetch), Custom(CustomIn));

#[derive(Debug)]
pub struct FetchInFetch {
	pub id:      Id,
	pub fetcher: FetcherArc,
	pub targets: Files,
	pub hashes:  Vec<u64>,
	pub scope:   Scope,
}

impl FetchInFetch {
	pub fn new(fetcher: impl Into<FetcherArc>, targets: Files) -> Self {
		let hashes = targets.hashes().collect();
		Self::hashed(fetcher, targets, hashes)
	}

	pub fn hashed(fetcher: impl Into<FetcherArc>, targets: Files, hashes: Vec<u64>) -> Self {
		Self { id: Id::ZERO, fetcher: fetcher.into(), targets, hashes, scope: Scope::default() }
	}

	pub fn scoped(mut self, context: &(impl ScopeLike + ?Sized)) -> Self {
		self.scope = context.scope().detached();
		self
	}
}

impl TaskIn for FetchInFetch {
	type Prog = FetchProg;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn scope(&self) -> &Scope { &self.scope }

	fn title(&self) -> Cow<'_, str> {
		format!("Run fetcher '{}' with {} target(s)", self.fetcher.name, self.targets.len()).into()
	}
}

impl From<FetchInFetch> for FetchJob {
	fn from(value: FetchInFetch) -> Self {
		Self { fetcher: value.fetcher, files: value.targets, scope: value.scope }
	}
}
