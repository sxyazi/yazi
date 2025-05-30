use std::{mem, time::Duration};

use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp, expand_path};
use yazi_macro::render;
use yazi_proxy::{CmpProxy, InputProxy, MgrProxy, TabProxy};
use yazi_shared::{Debounce, errors::InputError, event::CmdCow, url::Url};

use crate::tab::Tab;

struct Opt {
	target:      Url,
	source:      OptSource,
	interactive: bool,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			source: OptSource::Cd,
			interactive: c.bool("interactive"),
			..Self::from(c.take_first_url().unwrap_or_default())
		}
	}
}

impl From<Url> for Opt {
	fn from(target: Url) -> Self { Self::from((target, OptSource::Cd)) }
}

impl From<(Url, OptSource)> for Opt {
	fn from((mut target, source): (Url, OptSource)) -> Self {
		if target.is_regular() {
			target = Url::from(expand_path(&target));
		}
		Self { target, source, interactive: false }
	}
}

impl Tab {
	#[yazi_codegen::command]
	pub fn cd(&mut self, opt: Opt) {
		if !self.try_escape_visual() {
			return;
		}

		if opt.interactive {
			return self.cd_interactive();
		}

		if opt.target == *self.cwd() {
			return;
		}

		// Take parent to history
		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.url.to_owned(), rep);
		}

		// Current
		let rep = self.history.remove_or(&opt.target);
		let rep = mem::replace(&mut self.current, rep);
		if rep.url.is_regular() {
			self.history.insert(rep.url.to_owned(), rep);
		}

		// Parent
		if let Some(parent) = opt.target.parent_url() {
			self.parent = Some(self.history.remove_or(&parent));
		}

		// Backstack
		if opt.source.big_jump() && opt.target.is_regular() {
			self.backstack.push(&opt.target);
		}

		Pubsub::pub_from_cd(self.id, self.cwd());
		self.hover(None);

		MgrProxy::refresh();
		render!();
	}

	fn cd_interactive(&mut self) {
		let input = InputProxy::show(InputCfg::cd());

		tokio::spawn(async move {
			let rx = Debounce::new(UnboundedReceiverStream::new(input), Duration::from_millis(50));
			pin!(rx);

			while let Some(result) = rx.next().await {
				match result {
					Ok(s) => {
						let url = Url::from(expand_path(s));

						let Ok(file) = File::new(url.clone()).await else { return };
						if file.is_dir() {
							return TabProxy::cd(&url);
						}

						if let Some(p) = url.parent_url() {
							FilesOp::Upserting(p, [(url.urn_owned(), file)].into()).emit();
						}
						TabProxy::reveal(&url);
					}
					Err(InputError::Completed(before, ticket)) => {
						CmpProxy::trigger(&before, ticket);
					}
					_ => break,
				}
			}
		});
	}
}

// --- OptSource
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum OptSource {
	Cd,
	Reveal,
	Enter,
	Leave,
	Forward,
	Back,
}

impl OptSource {
	#[inline]
	fn big_jump(self) -> bool { self == Self::Cd || self == Self::Reveal }
}
