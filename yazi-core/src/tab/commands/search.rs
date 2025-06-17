use std::{borrow::Cow, mem, time::Duration};

use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tracing::error;
use yazi_config::popup::InputCfg;
use yazi_fs::{FilesOp, cha::Cha};
use yazi_plugin::external;
use yazi_proxy::{AppProxy, InputProxy, MgrProxy, TabProxy, options::{SearchOpt, SearchOptVia}};

use crate::tab::Tab;

impl Tab {
	pub fn search(&mut self, opt: impl TryInto<SearchOpt>) {
		let Ok(mut opt): Result<SearchOpt, _> = opt.try_into() else {
			return AppProxy::notify_error("Invalid `search` option", "Failed to parse search option");
		};

		if let Some(handle) = self.search.take() {
			handle.abort();
		}

		let mut input =
			InputProxy::show(InputCfg::search(opt.via.into_str()).with_value(&*opt.subject));
		tokio::spawn(async move {
			if let Some(Ok(subject)) = input.recv().await {
				opt.subject = Cow::Owned(subject);
				TabProxy::search_do(opt);
			}
		});
	}

	pub fn search_do(&mut self, opt: impl TryInto<SearchOpt>) {
		let Ok(opt): Result<SearchOpt, _> = opt.try_into() else {
			return error!("Failed to parse search option for `search_do`");
		};

		if let Some(handle) = self.search.take() {
			handle.abort();
		}

		let cwd = self.cwd().to_search(opt.subject.as_ref());
		let hidden = self.pref.show_hidden;

		self.search = Some(tokio::spawn(async move {
			let rx = match opt.via {
				SearchOptVia::Rg => external::rg(external::RgOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject.into_owned(),
					args: opt.args,
				}),
				SearchOptVia::Rga => external::rga(external::RgaOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject.into_owned(),
					args: opt.args,
				}),
				SearchOptVia::Fd => external::fd(external::FdOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject.into_owned(),
					args: opt.args,
				}),
			}?;

			let rx = UnboundedReceiverStream::new(rx).chunks_timeout(5000, Duration::from_millis(500));
			pin!(rx);

			let ((), ticket) = (TabProxy::cd(&cwd), FilesOp::prepare(&cwd));
			while let Some(chunk) = rx.next().await {
				FilesOp::Part(cwd.clone(), chunk, ticket).emit();
			}
			FilesOp::Done(cwd, Cha::default(), ticket).emit();

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
			MgrProxy::refresh();
		}
	}
}
