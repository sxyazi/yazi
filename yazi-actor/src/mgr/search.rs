use std::{borrow::Cow, time::Duration};

use anyhow::Result;
use tokio::pin;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_config::popup::InputCfg;
use yazi_core::mgr::{CdSource, SearchVia};
use yazi_fs::{FilesOp, cha::Cha};
use yazi_macro::{act, input, succ};
use yazi_parser::{VoidForm, mgr::SearchForm};
use yazi_plugin::external;
use yazi_proxy::MgrProxy;
use yazi_scheduler::NotifyProxy;
use yazi_shared::{data::Data, url::{AsUrl, UrlLike}};
use yazi_widgets::input::InputEvent;

use crate::{Actor, Ctx};

pub struct Search;

impl Actor for Search {
	type Form = SearchForm;

	const NAME: &str = "search";

	fn act(cx: &mut Ctx, Self::Form { mut opt }: Self::Form) -> Result<Data> {
		if let Some(handle) = cx.tab_mut().search.take() {
			handle.abort();
		}

		let mut input = input!(cx, InputCfg::search(opt.via.into()).with_value(&*opt.subject))?;

		tokio::spawn(async move {
			if let Some(InputEvent::Submit(subject)) = input.recv().await {
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
	type Form = SearchForm;

	const NAME: &str = "search_do";

	fn act(cx: &mut Ctx, Self::Form { opt }: Self::Form) -> Result<Data> {
		let tab = cx.tab_mut();
		if let Some(handle) = tab.search.take() {
			handle.abort();
		}

		let hidden = tab.pref.show_hidden;
		let r#in = opt.r#in.as_ref().map_or_else(|| tab.cwd().as_url(), |u| u.as_url());
		let Ok(cwd) = r#in.to_search(&opt.subject) else {
			succ!(NotifyProxy::push_warn("Search", "Only local filesystem searches are supported"));
		};

		tab.search = Some(tokio::spawn(async move {
			let rx = match opt.via {
				SearchVia::Rg => external::rg(external::RgOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject.into_owned(),
					args: opt.args,
				}),
				SearchVia::Rga => external::rga(external::RgaOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject.into_owned(),
					args: opt.args,
				}),
				SearchVia::Fd => external::fd(external::FdOpt {
					cwd: cwd.clone(),
					hidden,
					subject: opt.subject.into_owned(),
					args: opt.args,
				}),
			}?;

			let rx = UnboundedReceiverStream::new(rx).chunks_timeout(5000, Duration::from_millis(500));
			pin!(rx);

			let ((), ticket) = (MgrProxy::cd(&cwd, CdSource::Search), FilesOp::prepare(&cwd));
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
	type Form = VoidForm;

	const NAME: &str = "search_stop";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let tab = cx.tab_mut();
		if let Some(handle) = tab.search.take() {
			handle.abort();
		}

		if !tab.cwd().is_search() {
			succ!();
		}

		if let Some(u) = tab.backstack.current().cloned() {
			act!(mgr:cd, cx, (u, CdSource::Escape))
		} else {
			act!(mgr:cd, cx, (tab.cwd().to_regular()?, CdSource::Escape))
		}
	}
}
