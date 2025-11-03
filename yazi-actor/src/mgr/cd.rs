use std::{mem, time::Duration};

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp, path::expand_url};
use yazi_macro::{act, err, render, succ};
use yazi_parser::mgr::CdOpt;
use yazi_proxy::{CmpProxy, InputProxy, MgrProxy};
use yazi_shared::{Debounce, data::Data, errors::InputError, path::PathLike, url::{AsUrl, UrlBuf, UrlLike}};
use yazi_vfs::VfsFile;

use crate::{Actor, Ctx};

pub struct Cd;

impl Actor for Cd {
	type Options = CdOpt;

	const NAME: &str = "cd";

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
				tab.backstack.push(tab.current.url.as_url());
			}
			if opt.target.is_regular() {
				tab.backstack.push(opt.target.as_url());
			}
		}

		// Current
		let mut rep = tab.history.remove_or(&opt.target);

		// Only force reload if folder doesn't have cached ignore filters
		// If filters are cached, we can reuse the folder as-is (files are already
		// filtered) This avoids the race condition where cached unfiltered files
		// appear before plugin runs
		if rep.files.ignore_filter().is_none() {
			rep.cha = Default::default();
			rep.files.update_ioerr();
			rep.stage = Default::default();
		}
		let rep = mem::replace(&mut tab.current, rep);
		tab.history.insert(rep.url.to_owned(), rep);

		// Parent
		if let Some(parent) = opt.target.parent() {
			let mut parent_folder = tab.history.remove_or(parent);
			// Only force parent reload if it doesn't have cached filters
			if parent_folder.files.ignore_filter().is_none() {
				parent_folder.cha = Default::default();
				parent_folder.files.update_ioerr();
				parent_folder.stage = Default::default();
			}
			tab.parent = Some(parent_folder);
		}
		err!(Pubsub::pub_after_cd(tab.id, tab.cwd()));
		act!(mgr:hidden, cx)?;
		act!(mgr:sort, cx)?;
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
						let Ok(url) = UrlBuf::try_from(s).map(expand_url) else { return };

						let Ok(file) = File::new(&url).await else { return };
						if file.is_dir() {
							return MgrProxy::cd(&url);
						}

						if let Some(p) = url.parent() {
							FilesOp::Upserting(p.into(), [(url.urn().owned(), file)].into()).emit();
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
