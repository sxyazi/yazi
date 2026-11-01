use std::ffi::OsString;

use clap::CommandFactory;
use clap_complete::{CompletionCandidate, Shell};
use tokio::{runtime::Handle, task};
use yazi_shim::clap_complete::Completion;

use crate::{Args, package::Package};

#[derive(clap::Args)]
pub(crate) struct Cmp {
	shell: Shell,
	#[arg(long)]
	index: Option<usize>,
	#[arg(last = true, required = true)]
	words: Vec<OsString>,
}

impl Cmp {
	pub(crate) fn complete(self) -> anyhow::Result<()> {
		task::block_in_place(|| Completion::complete(self.shell, self.index, self.words, Args::command))
	}

	pub(crate) fn packages() -> Vec<CompletionCandidate> {
		let pkg = Handle::current().block_on(Package::load()).unwrap_or_default();

		pkg.plugins.into_iter().chain(pkg.flavors).map(|d| CompletionCandidate::new(d.r#use)).collect()
	}
}
