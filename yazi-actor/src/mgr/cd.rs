use std::{mem, time::Duration};

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp, expand_path};
use yazi_macro::{act, err, render, succ};
use yazi_parser::tab::CdOpt;
use yazi_proxy::{CmpProxy, InputProxy, MgrProxy};
use yazi_shared::{Debounce, errors::InputError, event::Data, url::Url};

use crate::{Actor, Ctx};

pub struct Cd;

impl Actor for Cd {
	type Options = CdOpt;

	const NAME: &'static str = "cd";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;
		if opt.interactive {
			return Self::cd_interactive();
		}

		let tab = cx.tab_mut();
		if opt.target == *tab.cwd() {
			succ!();
		}

		// Take parent to history
		if let Some(rep) = tab.parent.take() {
			tab.history.insert(rep.url.to_owned(), rep);
		}

		// Backstack
		if opt.source.big_jump() {
			if tab.current.url.is_regular() {
				tab.backstack.push(&tab.current.url);
			}
			if opt.target.is_regular() {
				tab.backstack.push(&opt.target);
			}
		}

		// Current
		let rep = tab.history.remove_or(&opt.target);
		let rep = mem::replace(&mut tab.current, rep);
		if rep.url.is_regular() {
			tab.history.insert(rep.url.to_owned(), rep);
		}

		// Parent
		if let Some(parent) = opt.target.parent_url() {
			tab.parent = Some(tab.history.remove_or(&parent));
		}

		err!(Pubsub::pub_after_cd(tab.id, tab.cwd()));
		act!(mgr:hover, cx)?;
		act!(mgr:refresh, cx)?;
		succ!(render!());
	}
}

impl Cd {
	fn cd_interactive() -> Result<Data> {
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
							return MgrProxy::cd(&url);
						}

						if let Some(p) = url.parent_url() {
							FilesOp::Upserting(p, [(url.urn_owned(), file)].into()).emit();
						}
						MgrProxy::reveal(&url);
					}
					Err(InputError::Completed(before, ticket)) => {
						CmpProxy::trigger(&before, ticket);
					}
					_ => break,
				}
			}
		});
		succ!();
	}
}
