use std::{mem, time::Duration};

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp, path::{clean_url, expand_url}};
use yazi_macro::{act, err, render, succ};
use yazi_parser::mgr::CdOpt;
use yazi_proxy::{CmpProxy, InputProxy, MgrProxy};
use yazi_shared::{Debounce, data::Data, errors::InputError, url::{AsUrl, UrlBuf, UrlLike}};
use yazi_vfs::{VfsFile, provider};

use crate::{Actor, Ctx};

pub struct Cd;

impl Actor for Cd {
	type Options = CdOpt;

	const NAME: &str = "cd";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;
		if opt.interactive {
			return Self::cd_interactive(cx);
		}

		let tab = cx.tab_mut();
		if opt.target == *tab.cwd() {
			succ!();
		}

		// Take parent to history
		if let Some(t) = tab.parent.take() {
			tab.history.insert(t.url.clone(), t);
		}

		// Current
		let rep = tab.history.remove_or(&opt.target);
		let rep = mem::replace(&mut tab.current, rep);
		tab.history.insert(rep.url.clone(), rep);

		// Parent
		if let Some(parent) = opt.target.parent() {
			tab.parent = Some(tab.history.remove_or(parent));
		}

		err!(Pubsub::pub_after_cd(tab.id, tab.cwd()));
		act!(mgr:displace, cx)?;
		act!(mgr:hidden, cx)?;
		act!(mgr:sort, cx).ok();
		act!(mgr:hover, cx)?;
		act!(mgr:refresh, cx)?;
		act!(mgr:stash, cx, opt).ok();
		succ!(render!());
	}
}

impl Cd {
	fn cd_interactive(cx: &Ctx) -> Result<Data> {
		let input = InputProxy::show(InputCfg::cd(cx.cwd().as_url()));

		tokio::spawn(async move {
			let rx = Debounce::new(UnboundedReceiverStream::new(input), Duration::from_millis(50));
			pin!(rx);

			while let Some(result) = rx.next().await {
				match result {
					Ok(s) => {
						let Ok(url) = UrlBuf::try_from(s).map(expand_url) else { return };
						let Ok(url) = provider::absolute(&url).await else { return };
						let url = clean_url(url);

						let Ok(file) = File::new(&url).await else { return };
						if file.is_dir() {
							return MgrProxy::cd(&url);
						}

						if let Some(p) = url.parent() {
							FilesOp::Upserting(p.into(), [(url.urn().into(), file)].into()).emit();
						}
						MgrProxy::reveal(url);
					}
					Err(InputError::Completed(before, ticket)) => {
						CmpProxy::trigger(before, ticket);
					}
					_ => break,
				}
			}
		});
		succ!();
	}
}
