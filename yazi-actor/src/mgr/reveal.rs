use anyhow::Result;
use yazi_fs::{File, FilesOp};
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::RevealOpt;
use yazi_shared::{data::Data, url::{AsUrl, UrlLike}};

use crate::{Actor, Ctx};

pub struct Reveal;

impl Actor for Reveal {
	type Options = RevealOpt;

	const NAME: &str = "reveal";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		match cx.cwd().is_search() {
			true => Self::reveal_search(cx, opt),
			false => Self::reveal_regular(cx, opt),
		}
	}
}

impl Reveal {
	fn reveal_regular(cx: &mut Ctx, opt: RevealOpt) -> Result<Data> {
		let Some((parent, child)) = opt.target.pair() else { succ!() };

		// Cd to the parent directory
		act!(mgr:cd, cx, (parent, opt.source))?;

		// Try to hover over the child file
		let tab = cx.tab_mut();
		render!(tab.current.hover(child));

		// If the child is not hovered, which means it doesn't exist,
		// create a dummy file
		if !opt.no_dummy && tab.hovered().is_none_or(|f| child != f.urn()) {
			let op = FilesOp::Creating(parent.into(), vec![File::from_dummy(&opt.target, None)]);
			tab.current.update_pub(tab.id, op);
		}

		// Now, we can safely hover over the target
		act!(mgr:hover, cx, Some(child.into()))?;

		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx)?;
		succ!();
	}

	fn reveal_search(cx: &mut Ctx, opt: RevealOpt) -> Result<Data> {
		let cwd = cx.cwd().clone();
		let tab = cx.tab_mut();
		let pos = tab.current.files.iter().position(|f| f.url == opt.target);

		if let Some(pos) = pos {
			let delta = pos as isize - tab.current.cursor as isize;
			tab.current.arrow(delta);
		} else if !opt.no_dummy {
			let op = FilesOp::Creating(cwd, vec![File::from_dummy(&opt.target, None)]);
			tab.current.update_pub(tab.id, op);

			if let Some(pos) = tab.current.files.iter().position(|f| f.url == opt.target) {
				let delta = pos as isize - tab.current.cursor as isize;
				tab.current.arrow(delta);
			}
		}

		act!(mgr:hover, cx, None)?;
		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx)?;
		succ!();
	}
}
