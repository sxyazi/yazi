use std::{mem, time::Duration};

use tokio::{fs, pin};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_dds::Pubsub;
use yazi_proxy::{CompletionProxy, InputProxy, ManagerProxy, TabProxy};
use yazi_shared::{event::{Cmd, Data}, fs::{expand_path, Url}, render, Debounce, InputError};

use crate::tab::Tab;

pub struct Opt {
	target:      Url,
	interactive: bool,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		let mut target = c.take_first().and_then(Data::into_url).unwrap_or_default();
		if target.is_regular() {
			target.set_path(expand_path(&target))
		}

		Self { target, interactive: c.bool("interactive") }
	}
}
impl From<Url> for Opt {
	fn from(target: Url) -> Self { Self { target, interactive: false } }
}

impl Tab {
	pub fn cd(&mut self, opt: impl Into<Opt>) {
		if !self.try_escape_visual() {
			return;
		}

		let opt = opt.into() as Opt;
		if opt.interactive {
			return self.cd_interactive();
		}

		if opt.target == *self.cwd() {
			return;
		}

		// Take parent to history
		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.loc.url().clone(), rep);
		}

		// Current
		let rep = self.history.remove_or(&opt.target);
		let rep = mem::replace(&mut self.current, rep);
		if rep.loc.is_regular() {
			self.history.insert(rep.loc.url().clone(), rep);
		}

		// Parent
		if let Some(parent) = opt.target.parent_url() {
			self.parent = Some(self.history.remove_or(&parent));
		}

		// Backstack
		if opt.target.is_regular() {
			self.backstack.push(opt.target.clone());
		}

		Pubsub::pub_from_cd(self.idx, self.cwd());
		ManagerProxy::refresh();
		render!();
	}

	fn cd_interactive(&mut self) {
		tokio::spawn(async move {
			let rx = InputProxy::show(InputCfg::cd());

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(result) = rx.next().await {
				match result {
					Ok(s) => {
						let u = Url::from(expand_path(s));
						let Ok(meta) = fs::metadata(&u).await else {
							return;
						};

						if meta.is_dir() {
							TabProxy::cd(&u);
						} else {
							TabProxy::reveal(&u);
						}
					}
					Err(InputError::Completed(before, ticket)) => {
						CompletionProxy::trigger(&before, ticket);
					}
					_ => break,
				}
			}
		});
	}
}
