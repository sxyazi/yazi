use std::{mem, time::Duration};

use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp, expand_path};
use yazi_macro::{err, render};
use yazi_proxy::{CmpProxy, InputProxy, MgrProxy, TabProxy};
use yazi_shared::{Debounce, errors::InputError, event::CmdCow, url::Url};

use crate::tab::Tab;

struct Opt {
	target:      Url,
	interactive: bool,
	source:      CdSource,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		let mut target = c.take_first_url().unwrap_or_default();
		if target.is_regular() && !c.bool("raw") {
			target = Url::from(expand_path(target));
		}
		Self { target, interactive: c.bool("interactive"), source: CdSource::Cd }
	}
}

impl From<(Url, CdSource)> for Opt {
	fn from((target, source): (Url, CdSource)) -> Self { Self { target, interactive: false, source } }
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

		// Backstack
		if opt.source.big_jump() {
			if self.current.url.is_regular() {
				self.backstack.push(&self.current.url);
			}
			if opt.target.is_regular() {
				self.backstack.push(&opt.target);
			}
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

		err!(Pubsub::pub_from_cd(self.id, self.cwd()));
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
pub enum CdSource {
	Tab,
	Cd,
	Reveal,
	Enter,
	Leave,
	Forward,
	Back,
}

impl CdSource {
	#[inline]
	fn big_jump(self) -> bool { self == Self::Cd || self == Self::Reveal }
}
