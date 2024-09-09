use std::{mem, time::Duration};

use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tracing::error;
use yazi_config::popup::InputCfg;
use yazi_plugin::external;
use yazi_proxy::{options::{SearchOpt, SearchOptVia}, AppProxy, InputProxy, ManagerProxy, TabProxy};
use yazi_shared::fs::{Cha, FilesOp};

use crate::tab::Tab;

impl Tab {
	pub fn search(&mut self, opt: impl TryInto<SearchOpt>) {
		let Ok(mut opt) = opt.try_into() else {
			return AppProxy::notify_error("Invalid `search` option", "Failed to parse search option");
		};

		if opt.via == SearchOptVia::None {
			return self.search_stop();
		}

		if let Some(handle) = self.search.take() {
			handle.abort();
		}

		tokio::spawn(async move {
			let mut input =
				InputProxy::show(InputCfg::search(&opt.via.to_string()).with_value(opt.subject));

			if let Some(Ok(subject)) = input.recv().await {
				opt.subject = subject;
				TabProxy::search_do(opt);
			}
		});
	}

	pub fn search_do(&mut self, opt: impl TryInto<SearchOpt>) {
		let Ok(opt) = opt.try_into() else {
			return error!("Failed to parse search option for `search_do`");
		};

		if let Some(handle) = self.search.take() {
			handle.abort();
		}

		let cwd = self.cwd().to_search(&opt.subject);
		let hidden = self.conf.show_hidden;

		self.search = Some(tokio::spawn(async move {
			let rx = if opt.via == SearchOptVia::Rg {
				external::rg(external::RgOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject,
					args: opt.args,
				})
			} else {
				external::fd(external::FdOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject,
					args: opt.args,
				})
			}?;

			let rx = UnboundedReceiverStream::new(rx).chunks_timeout(1000, Duration::from_millis(300));
			pin!(rx);

			let ((), ticket) = (TabProxy::cd(&cwd), FilesOp::prepare(&cwd));
			while let Some(chunk) = rx.next().await {
				FilesOp::Part(cwd.clone(), chunk, ticket).emit();
			}
			FilesOp::Done(cwd, Cha::dummy(), ticket).emit();

			Ok(())
		}));
	}

	pub(super) fn search_stop(&mut self) {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}
		if self.cwd().is_search() {
			let rep = self.history.remove_or(&self.cwd().to_regular());
			drop(mem::replace(&mut self.current, rep));
			ManagerProxy::refresh();
		}
	}
}
