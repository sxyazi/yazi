use anyhow::Result;
use yazi_dds::Pubsub;
use yazi_macro::{err, render, succ, tab};
use yazi_parser::mgr::HoverForm;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Hover;

impl Actor for Hover {
	type Form = HoverForm;

	const NAME: &str = "hover";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let tab = tab!(cx);

		// Parent should always track CWD
		if let Some(p) = &mut tab.parent {
			render!(p.repos(tab.current.url.try_strip_prefix(&p.url).ok()));
		}

		// Repos CWD
		render!(tab.current.repos(form.urn.as_ref().map(Into::into)));

		// Turn on tracing
		if let (Some(h), Some(u)) = (tab.hovered(), form.urn)
			&& h.urn() == u
		{
			// `hover(Some)` occurs after user actions, such as create, rename, reveal, etc.
			// At this point, it's intuitive to track the file location regardless.
			tab.current.trace = Some(u.clone());
			cx.tasks.scheduler.behavior.reset();
		}

		// Publish through DDS
		let tab = tab!(cx);
		err!(Pubsub::pub_after_hover(tab.id, tab.hovered().map(|h| &h.url)));
		succ!();
	}
}
