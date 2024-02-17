use std::{mem, time::Duration};

use tokio::{fs, pin};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_shared::{emit, event::Cmd, fs::{expand_path, Url}, render, Debounce, InputError, Layer};

use crate::{completion::Completion, input::Input, manager::Manager, tab::Tab};

pub struct Opt {
	target:      Url,
	interactive: bool,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		let mut target = Url::from(c.take_first().unwrap_or_default());
		if target.is_regular() {
			target.set_path(expand_path(&target))
		}

		Self { target, interactive: c.named.contains_key("interactive") }
	}
}
impl From<Url> for Opt {
	fn from(target: Url) -> Self { Self { target, interactive: false } }
}

impl Tab {
	#[inline]
	pub fn _cd(target: &Url) {
		emit!(Call(Cmd::args("cd", vec![target.to_string()]), Layer::Manager));
	}

	pub fn cd(&mut self, opt: impl Into<Opt>) {
		if !self.try_escape_visual() {
			return;
		}

		let opt = opt.into() as Opt;
		if opt.interactive {
			return self.cd_interactive();
		}

		if self.current.cwd == opt.target {
			return;
		}

		// Take parent to history
		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Current
		let rep = self.history_new(&opt.target);
		let rep = mem::replace(&mut self.current, rep);
		if rep.cwd.is_regular() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Parent
		if let Some(parent) = opt.target.parent_url() {
			self.parent = Some(self.history_new(&parent));
		}

		// Backstack
		if opt.target.is_regular() {
			self.backstack.push(opt.target.clone());
		}

		Manager::_refresh();
		render!();
	}

	fn cd_interactive(&mut self) {
		tokio::spawn(async move {
			let rx = Input::_show(InputCfg::cd());

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
							Tab::_cd(&u);
						} else {
							Tab::_reveal(&u);
						}
					}
					Err(InputError::Completed(before, ticket)) => {
						Completion::_trigger(&before, ticket);
					}
					_ => break,
				}
			}
		});
	}
}
