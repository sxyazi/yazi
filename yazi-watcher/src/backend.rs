use notify::Result;
use tokio::sync::mpsc;

use crate::{Reporter, WATCHED, Watchee, local::{self, LINKED, Linked}, remote};

pub(crate) struct Backend {
	local:               local::Local,
	remote:              remote::Remote,
	pub(super) reporter: Reporter,
}

impl Backend {
	pub(crate) fn serve() -> Self {
		#[cfg(any(target_os = "linux", target_os = "macos"))]
		yazi_fs::mounts::Partitions::monitor(&yazi_fs::mounts::PARTITIONS, || {
			yazi_proxy::MgrProxy::watch();
			yazi_proxy::MgrProxy::refresh();
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

	pub(super) fn watch(&mut self, watchee: &mut Watchee) -> Result<()> {
		match watchee {
			Watchee::Local(..) => self.local.watch(watchee),
			Watchee::Remote(_) => self.remote.watch(watchee),
		}
	}

	pub(super) fn unwatch(&mut self, watchee: &Watchee) -> Result<()> {
		match watchee {
			Watchee::Local(..) => self.local.unwatch(watchee),
			Watchee::Remote(_) => self.remote.unwatch(watchee),
		}
	}

	pub(super) async fn sync(self) -> Self {
		Linked::sync(&LINKED, &WATCHED).await;
		self
	}
}
