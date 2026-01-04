use std::{borrow::Cow, time::Duration};

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_fs::{FilesOp, cha::Cha};
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, mgr::{CdSource, SearchOpt, SearchOptVia}};
use yazi_plugin::external;
use yazi_proxy::{AppProxy, InputProxy, MgrProxy};
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Search;

impl Actor for Search {
	type Options = SearchOpt;

	const NAME: &str = "search";

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

	const NAME: &str = "search_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		if let Some(handle) = tab.search.take() {
			handle.abort();
		}

		let hidden = tab.pref.show_hidden;
		let Ok(cwd) = tab.cwd().to_search(&opt.subject) else {
			succ!(AppProxy::notify_warn("Search", "Only local filesystem searches are supported"));
		};

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
	type Options = VoidOpt;

	const NAME: &str = "search_stop";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let tab = cx.tab_mut();
		if let Some(handle) = tab.search.take() {
			handle.abort();
		}

		if tab.cwd().is_search() {
			act!(mgr:cd, cx, (tab.cwd().to_regular()?, CdSource::Escape))?;
		}

		succ!();
	}
}
