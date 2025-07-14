use anyhow::Result;
use yazi_fs::{File, FilesOp};
use yazi_macro::{act, succ};
use yazi_parser::tab::RevealOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Reveal;

impl Actor for Reveal {
	type Options = RevealOpt;

	const NAME: &'static str = "reveal";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let Some((parent, child)) = opt.target.pair() else { succ!() };

		// Cd to the parent directory
		act!(mgr:cd, cx, (parent.clone(), opt.source))?;

		// Try to hover on the child file
		let tab = cx.tab_mut();
		tab.current.hover(child.as_urn());

		// If the child is not hovered, which means it doesn't exist,
		// create a dummy file
		if !opt.no_dummy && tab.hovered().is_none_or(|f| &child != f.urn()) {
			let op = FilesOp::Creating(parent, vec![File::from_dummy(opt.target.clone(), None)]);
			tab.current.update_pub(tab.id, op);
		}

		// Now, we can safely hover on the target
		act!(mgr:hover, cx, Some(opt.target))?;

		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx)?;
		succ!();
	}
}
