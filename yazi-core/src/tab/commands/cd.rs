use std::{mem, time::Duration};

use tokio::{fs, pin};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::{keymap::{Exec, KeymapLayer}, popup::InputOpt};
use yazi_shared::{expand_path, Debounce, InputError, Url};

use crate::{completion::Completion, emit, manager::Manager, tab::Tab};

pub struct Opt {
	target:      Url,
	interactive: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		let mut target = Url::from(e.args.first().map(|s| s.as_str()).unwrap_or(""));
		if target.is_regular() {
			target.set_path(expand_path(&target))
		}

		Self { target, interactive: e.named.contains_key("interactive") }
	}
}
impl From<Url> for Opt {
	fn from(target: Url) -> Self { Self { target, interactive: false } }
}

impl Tab {
	#[inline]
	pub fn _cd(target: &Url) {
		emit!(Call(Exec::call("cd", vec![target.to_string()]).vec(), KeymapLayer::Manager));
	}

	pub fn cd(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;
		if opt.interactive {
			return self.cd_interactive(opt);
		}

		if self.current.cwd == opt.target {
			return false;
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
		true
	}

	fn cd_interactive(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;

		tokio::spawn(async move {
			let rx = emit!(Input(InputOpt::cd().with_value(opt.target.to_string_lossy())));

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
		false
	}
}
