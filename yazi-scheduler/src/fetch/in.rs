use std::{borrow::Cow, sync::Arc};

use yazi_config::plugin::Fetcher;
use yazi_runner::fetcher::FetchJob;
use yazi_shared::Id;

use crate::{TaskIn, fetch::FetchProg};

#[derive(Debug)]
pub(crate) struct FetchIn {
	pub(crate) id:      Id,
	pub(crate) fetcher: Arc<Fetcher>,
	pub(crate) targets: Vec<yazi_fs::File>,
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
