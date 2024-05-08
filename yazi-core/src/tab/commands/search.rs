use std::{fmt::Display, mem, time::Duration};

use anyhow::bail;
use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_plugin::external;
use yazi_proxy::{AppProxy, InputProxy, ManagerProxy, TabProxy};
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
	pub args:  Vec<String>,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self {
			type_: c.take_first_str().unwrap_or_default().into(),
			args:  shell_words::split(c.str("args").unwrap_or_default()).map_err(|_| ())?,
		})
	}
}

impl Tab {
	pub fn search(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else {
			return AppProxy::notify_error("Invalid `search` option", "Failed to parse search option");
		};

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
				external::rg(external::RgOpt { cwd: cwd.clone(), hidden, subject, args: opt.args })
			} else {
				external::fd(external::FdOpt { cwd: cwd.clone(), hidden, subject, args: opt.args })
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
