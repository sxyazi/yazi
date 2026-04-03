use yazi_config::plugin::Fetcher;
use yazi_runner::fetcher::FetchJob;
use yazi_shared::Id;

#[derive(Debug)]
pub(crate) struct FetchIn {
	pub(crate) id:      Id,
	pub(crate) plugin:  &'static Fetcher,
	pub(crate) targets: Vec<yazi_fs::File>,
}

impl From<FetchIn> for FetchJob {
	fn from(value: FetchIn) -> Self { Self { action: &value.plugin.run, files: value.targets } }
}

impl FetchIn {
	pub(crate) fn id(&self) -> Id { self.id }
}
