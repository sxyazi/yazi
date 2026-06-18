use std::borrow::Cow;

use yazi_config::plugin::FetcherArc;
use yazi_runner::fetcher::FetchJob;
use yazi_shared::id::Id;

use crate::{TaskIn, fetch::FetchProg};

#[derive(Debug)]
pub(crate) struct FetchIn {
	pub(crate) id:      Id,
	pub(crate) fetcher: FetcherArc,
	pub(crate) targets: Vec<yazi_fs::file::File>,
}

impl TaskIn for FetchIn {
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

impl From<FetchIn> for FetchJob {
	fn from(value: FetchIn) -> Self { Self { fetcher: value.fetcher, files: value.targets } }
}
