use std::{borrow::Cow, mem, time::Duration};

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_fs::{FilesOp, cha::Cha};
use yazi_macro::{act, succ};
use yazi_plugin::external;
use yazi_proxy::{InputProxy, MgrProxy, options::{SearchOpt, SearchOptVia}};
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Search;

impl Actor for Search {
	type Options = SearchOpt;

	const NAME: &'static str = "search";

	fn act(cx: &mut Ctx, mut opt: Self::Options) -> Result<Data> {
		if let Some(handle) = cx.tab_mut().search.take() {
			handle.abort();
		}

		let mut input =
			InputProxy::show(InputCfg::search(opt.via.into_str()).with_value(&*opt.subject));

		tokio::spawn(async move {
			if let Some(Ok(subject)) = input.recv().await {
				opt.subject = Cow::Owned(subject);
				MgrProxy::search_do(opt);
			}
		});
		succ!();
	}
}

// --- Do
pub struct SearchDo;

impl Actor for SearchDo {
	type Options = SearchOpt;

	const NAME: &'static str = "search_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		if let Some(handle) = tab.search.take() {
			handle.abort();
		}

		let cwd = tab.cwd().to_search(opt.subject.as_ref());
		let hidden = tab.pref.show_hidden;

		tab.search = Some(tokio::spawn(async move {
			let rx = match opt.via {
				SearchOptVia::Rg => external::rg(external::RgOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject.into_owned(),
					args: opt.args,
				}),
				SearchOptVia::Rga => external::rga(external::RgaOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject.into_owned(),
					args: opt.args,
				}),
				SearchOptVia::Fd => external::fd(external::FdOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject.into_owned(),
					args: opt.args,
				}),
			}?;

			let rx = UnboundedReceiverStream::new(rx).chunks_timeout(5000, Duration::from_millis(500));
			pin!(rx);

			let ((), ticket) = (MgrProxy::cd(&cwd), FilesOp::prepare(&cwd));
			while let Some(chunk) = rx.next().await {
				FilesOp::Part(cwd.clone(), chunk, ticket).emit();
			}
			FilesOp::Done(cwd, Cha::default(), ticket).emit();

			Ok(())
		}));

		succ!();
	}
}

// --- Stop
pub struct SearchStop;

impl Actor for SearchStop {
	type Options = ();

	const NAME: &'static str = "search_stop";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		if let Some(handle) = tab.search.take() {
			handle.abort();
		}

		if tab.cwd().is_search() {
			let rep = tab.history.remove_or(&tab.cwd().to_regular());
			drop(mem::replace(&mut tab.current, rep));
			act!(mgr:refresh, cx)?;
		}
		succ!();
	}
}
