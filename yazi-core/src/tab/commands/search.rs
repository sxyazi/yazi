use std::{fmt::Display, mem, time::Duration};

use anyhow::bail;
use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_plugin::external;
use yazi_proxy::{InputProxy, ManagerProxy, TabProxy};
use yazi_shared::{event::Cmd, fs::FilesOp, render};

use crate::tab::Tab;

#[derive(PartialEq, Eq)]
pub enum OptType {
	None,
	Rg,
	Fd,
}

impl From<String> for OptType {
	fn from(value: String) -> Self {
		match value.as_str() {
			"rg" => Self::Rg,
			"fd" => Self::Fd,
			_ => Self::None,
		}
	}
}

impl Display for OptType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::Rg => "rg",
			Self::Fd => "fd",
			Self::None => "none",
		})
	}
}

pub struct Opt {
	pub type_: OptType,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self { Self { type_: c.take_first_str().unwrap_or_default().into() } }
}

impl Tab {
	pub fn search(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if opt.type_ == OptType::None {
			return self.search_stop();
		}

		if let Some(handle) = self.search.take() {
			handle.abort();
		}

		let mut cwd = self.current.cwd.clone();
		let hidden = self.conf.show_hidden;

		self.search = Some(tokio::spawn(async move {
			let mut input = InputProxy::show(InputCfg::search(&opt.type_.to_string()));
			let Some(Ok(subject)) = input.recv().await else { bail!("") };

			cwd = cwd.into_search(subject.clone());
			let rx = if opt.type_ == OptType::Rg {
				external::rg(external::RgOpt { cwd: cwd.clone(), hidden, subject })
			} else {
				external::fd(external::FdOpt { cwd: cwd.clone(), hidden, glob: false, subject })
			}?;

			let rx = UnboundedReceiverStream::new(rx).chunks_timeout(1000, Duration::from_millis(300));
			pin!(rx);

			let ((), ticket) = (TabProxy::cd(&cwd), FilesOp::prepare(&cwd));
			while let Some(chunk) = rx.next().await {
				FilesOp::Part(cwd.clone(), chunk, ticket).emit();
			}
			FilesOp::Done(cwd, None, ticket).emit();
			Ok(())
		}));

		render!();
	}

	pub(super) fn search_stop(&mut self) {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}
		if self.current.cwd.is_search() {
			let rep = self.history_new(&self.current.cwd.to_regular());
			drop(mem::replace(&mut self.current, rep));
			ManagerProxy::refresh();
		}
	}
}
