use std::borrow::Cow;

use yazi_config::plugin::FetcherArc;
use yazi_runner::fetcher::FetchJob;
use yazi_shared::id::Id;

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

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Fetch(r#in) => r#in.title(),
			Self::Custom(r#in) => r#in.title(),
		}
	}
}

impl_from_in!(Fetch(FetchInFetch), Custom(CustomIn));

#[derive(Debug)]
pub(crate) struct FetchInFetch {
	pub(crate) id:      Id,
	pub(crate) fetcher: FetcherArc,
	pub(crate) targets: Vec<yazi_fs::file::File>,
}

impl TaskIn for FetchInFetch {
	type Prog = FetchProg;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		format!("Run fetcher '{}' with {} target(s)", self.fetcher.name, self.targets.len()).into()
	}
}

impl From<FetchInFetch> for FetchJob {
	fn from(value: FetchInFetch) -> Self {
		Self { fetcher: value.fetcher, files: value.targets.into() }
	}
}
