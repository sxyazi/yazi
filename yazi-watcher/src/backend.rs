use anyhow::Result;
use tokio::sync::mpsc;
use yazi_shared::url::AsUrl;

use crate::{Reporter, WATCHED, local::{self, LINKED, Linked}, remote};

pub(crate) struct Backend {
	local:               local::Local,
	remote:              remote::Remote,
	pub(super) reporter: Reporter,
}

impl Backend {
	pub(crate) fn serve() -> Self {
		#[cfg(any(target_os = "linux", target_os = "macos"))]
		yazi_fs::mounts::Partitions::monitor(&yazi_fs::mounts::PARTITIONS, || {
			yazi_macro::err!(yazi_dds::Pubsub::pub_after_mount())
		});

		let (local_tx, local_rx) = mpsc::unbounded_channel();
		let (remote_tx, remote_rx) = mpsc::unbounded_channel();
		let reporter = Reporter { local_tx, remote_tx };

		Self {
			local: local::Local::serve(local_rx, reporter.clone()),
			remote: remote::Remote::serve(remote_rx, reporter.clone()),
			reporter,
		}
	}

	pub(super) fn watch(&mut self, url: impl AsUrl) -> Result<()> {
		let url = url.as_url();
		if let Some(path) = url.as_local() {
			self.local.watch(path)?;
		} else {
			self.remote.watch(url)?;
		}

		Ok(())
	}

	pub(super) fn unwatch(&mut self, url: impl AsUrl) -> Result<()> {
		let url = url.as_url();
		if let Some(path) = url.as_local() {
			self.local.unwatch(path)?;
		} else {
			self.remote.unwatch(url)?;
		}

		Ok(())
	}

	pub(super) async fn sync(self) -> Self {
		Linked::sync(&LINKED, &WATCHED).await;
		self
	}
}
