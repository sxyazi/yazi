use std::{mem, time::Duration};

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_core::mgr::CdSource;
use yazi_dds::Pubsub;
use yazi_fs::{File, FilesOp, path::{clean_url, expand_url}};
use yazi_macro::{act, err, input, render, succ};
use yazi_parser::mgr::CdForm;
use yazi_proxy::{CmpProxy, MgrProxy};
use yazi_shared::{Debounce, data::Data, url::{AsUrl, UrlBuf, UrlLike}};
use yazi_vfs::{VfsFile, provider};
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct Cd;

impl Actor for Cd {
	type Form = CdForm;

	const NAME: &str = "cd";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;
		if form.interactive {
			return Self::cd_interactive(cx);
		} else if form.target == *cx.cwd() {
			succ!();
		}

		// Stash first so it's possible to access the original cwd in hooks
		act!(mgr:stash, cx, &form).ok();

		// Take parent to history
		let tab = cx.tab_mut();
		if let Some(t) = tab.parent.take() {
			tab.history.insert(t.url.clone(), t);
		}

		// Current
		let rep = tab.history.remove_or(&form.target);
		let rep = mem::replace(&mut tab.current, rep);
		tab.history.insert(rep.url.clone(), rep);

		// Parent
		if let Some(parent) = form.target.parent() {
			tab.parent = Some(tab.history.remove_or(parent));
		}

		err!(Pubsub::pub_after_cd(tab.id, tab.cwd()));
		act!(mgr:displace, cx)?;
		act!(mgr:hidden, cx).ok();
		act!(mgr:sort, cx).ok();
		act!(mgr:hover, cx)?;
		act!(mgr:refresh, cx)?;
		act!(app:title, cx).ok();
		succ!(render!());
	}
}

impl Cd {
	fn cd_interactive(cx: &mut Ctx) -> Result<Data> {
		let input = input!(cx, InputCfg::cd(cx.cwd().as_url()))?;

		tokio::spawn(async move {
			let rx = Debounce::new(UnboundedReceiverStream::new(input), Duration::from_millis(50));
			pin!(rx);

			while let Some(result) = rx.next().await {
				match result {
					InputEvent::Submit(s) => {
						let Ok(url) = UrlBuf::try_from(s).map(expand_url) else { return };
						let Ok(url) = provider::absolute(&url).await else { return };
						let url = clean_url(url);

						let Ok(file) = File::new(&url).await else { return };
						if file.is_dir() {
							return MgrProxy::cd(&url, CdSource::Cd);
						}

						if let Some(p) = url.parent() {
							FilesOp::Upserting(p.into(), [(url.urn().into(), file)].into()).emit();
						}
						MgrProxy::reveal(url);
					}
					InputEvent::Trigger(before, ticket) => {
						CmpProxy::trigger(before, ticket);
					}
					_ => break,
				}
			}
		});
		succ!();
	}
}
