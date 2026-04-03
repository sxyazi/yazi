use anyhow::Result;
use yazi_fs::{File, FilesOp};
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::RevealForm;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Reveal;

impl Actor for Reveal {
	type Form = RevealForm;

	const NAME: &str = "reveal";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some((parent, child)) = form.target.pair() else { succ!() };

		// Cd to the parent directory
		act!(mgr:cd, cx, (parent, form.source))?;

		// Try to hover over the child file
		let tab = cx.tab_mut();
		render!(tab.current.hover(child));

		// If the child is not hovered, which means it doesn't exist,
		// create a dummy file
		if !form.no_dummy && tab.hovered().is_none_or(|f| child != f.urn()) {
			let op = FilesOp::Creating(parent.into(), vec![File::from_dummy(&form.target, None)]);
			tab.current.update_pub(tab.id, op);
		}

		// Now, we can safely hover over the target
		act!(mgr:hover, cx, Some(child.into()))?;

		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx)?;
		succ!();
	}
}
