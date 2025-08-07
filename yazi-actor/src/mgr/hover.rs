use anyhow::Result;
use yazi_dds::Pubsub;
use yazi_macro::{err, render, succ, tab};
use yazi_parser::mgr::HoverOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Hover;

impl Actor for Hover {
	type Options = HoverOpt;

	const NAME: &str = "hover";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let tab = tab!(cx);

		// Parent should always track CWD
		if let Some(p) = &mut tab.parent {
			render!(p.repos(tab.current.url.strip_prefix(&p.url)));
		}

		// Repos CWD
		tab.current.repos(opt.urn.as_deref());

		// Turn on tracing
		if let (Some(h), Some(u)) = (tab.hovered(), opt.urn)
			&& *h.urn() == u
		{
			// `hover(Some)` occurs after user actions, such as create, rename, reveal, etc.
			// At this point, it's intuitive to track the location of the file regardless.
			tab.current.trace = Some(u.to_owned());
		}

		// Publish through DDS
		err!(Pubsub::pub_after_hover(tab.id, tab.hovered().map(|h| &h.url)));
		succ!();
	}
}
