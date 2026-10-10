use anyhow::Result;
use yazi_core::Ctx;
use yazi_fs::{file::File, op::FilesOp};
use yazi_macro::{render, succ};
use yazi_parser::mgr::RevealForm;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, act};

pub struct Reveal;

impl Actor for Reveal {
	type Form = RevealForm;

	const NAME: &str = "reveal";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let Some((trail, key)) = form.target.pair() else { succ!() };

		// Cd to the trail directory
		act!(mgr:cd, cx, (trail, form.source))?;

		// Try to hover over the child file
		let tab = cx.tab_mut();
		render!(tab.current.hover(key));

		// If the child is not hovered, which means it doesn't exist,
		// create a dummy file
		if !form.no_dummy && tab.hovered().is_none_or(|f| f.key() != key) {
			let op = FilesOp::Create(trail.into(), vec![File::from_dummy(&form.target, None)]);
			tab.current.update_pub(tab.id, op);
		}

		// Now, we can safely hover over the target
		act!(mgr:hover, cx, Some(key.into()))?;

		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx).ok();
		succ!();
	}
}
