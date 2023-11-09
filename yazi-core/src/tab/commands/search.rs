use std::{mem, time::Duration};

use anyhow::bail;
use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::keymap::{Exec, KeymapLayer};

use crate::{emit, external, files::FilesOp, input::InputOpt, tab::Tab};

impl Tab {
	pub fn search(&mut self, grep: bool) -> bool {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}

		let mut cwd = self.current.cwd.clone();
		let hidden = self.conf.show_hidden;

		self.search = Some(tokio::spawn(async move {
			let Some(Ok(subject)) = emit!(Input(InputOpt::top("Search:"))).recv().await else {
				bail!("")
			};

			cwd = cwd.into_search(subject.clone());
			let rx = if grep {
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
					emit!(Call(Exec::call("cd", vec![cwd.clone().to_string()]).vec(), KeymapLayer::Manager));
					first = false;
				}
				emit!(Files(FilesOp::Part(cwd.clone(), ticket, chunk)));
			}
			Ok(())
		}));
		true
	}

	pub fn search_stop(&mut self) -> bool {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}
		if self.current.cwd.is_search() {
			self.preview.reset(|l| l.is_image());

			let rep = self.history_new(&self.current.cwd.to_regular());
			drop(mem::replace(&mut self.current, rep));
			emit!(Refresh);
		}
		false
	}
}
