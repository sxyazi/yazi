use anyhow::Result;
use yazi_dds::Pubsub;
use yazi_macro::{act, err, render, succ};
use yazi_parser::tab::{HoverDoOpt, HoverOpt};
use yazi_shared::{event::Data, url::Urn};

use crate::{Actor, Ctx};

pub struct Hover;

impl Actor for Hover {
	type Options = HoverOpt;

	const NAME: &'static str = "hover";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if let Some(u) = opt.url {
			act!(mgr:hover_do, cx, u)?;
		} else {
			cx.current_mut().arrow(0);
		}

		// Publish through DDS
		let tab = cx.tab();
		err!(Pubsub::pub_after_hover(tab.id, tab.hovered().map(|h| &h.url)));
		succ!();
	}
}

// --- Do
pub struct HoverDo;

impl Actor for HoverDo {
	type Options = HoverDoOpt;

	const NAME: &'static str = "hover_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		// Hover on the file
		if let Ok(p) = opt.url.strip_prefix(cx.cwd()) {
			render!(cx.current_mut().hover(Urn::new(p)));
		}

		// Turn on tracing
		if cx.hovered().is_some_and(|h| h.url == opt.url) {
			// `hover(Some)` occurs after user actions, such as create, rename, reveal, etc.
			// At this point, it's intuitive to track the location of the file regardless.
			cx.current_mut().trace = Some(opt.url.urn_owned());
		}
		succ!();
	}
}
