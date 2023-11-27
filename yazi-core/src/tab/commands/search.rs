use std::{mem, time::Duration};

use anyhow::bail;
use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_shared::{emit, event::Exec, files::FilesOp};

use crate::{external, input::Input, manager::Manager, tab::Tab};

pub struct Opt {
	pub type_: OptType,
}

#[derive(PartialEq, Eq)]
pub enum OptType {
	None,
	Rg,
	Fd,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self {
			type_: match e.args.first().map(|s| s.as_str()) {
				Some("fd") => OptType::Fd,
				Some("rg") => OptType::Rg,
				_ => OptType::None,
			},
		}
	}
}

impl Tab {
	pub fn search(&mut self, opt: impl Into<Opt>) -> bool {
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
			let Some(Ok(subject)) = Input::_show(InputCfg::search()).recv().await else { bail!("") };

			cwd = cwd.into_search(subject.clone());
			let rx = if opt.type_ == OptType::Rg {
				external::rg(external::RgOpt { cwd: cwd.clone(), hidden, subject })
			} else {
				external::fd(external::FdOpt { cwd: cwd.clone(), hidden, glob: false, subject })
			}?;

			let rx = UnboundedReceiverStream::new(rx).chunks_timeout(1000, Duration::from_millis(300));
			pin!(rx);

			let ticket = FilesOp::prepare(&cwd);
			let mut first = true;
			while let Some(chunk) = rx.next().await {
				if first {
					Tab::_cd(&cwd);
					first = false;
				}
				emit!(Files(FilesOp::Part(cwd.clone(), ticket, chunk)));
			}
			Ok(())
		}));
		true
	}

	pub(super) fn search_stop(&mut self) -> bool {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}
		if self.current.cwd.is_search() {
			self.preview.reset_image();

			let rep = self.history_new(&self.current.cwd.to_regular());
			drop(mem::replace(&mut self.current, rep));
			Manager::_refresh();
		}
		false
	}
}
