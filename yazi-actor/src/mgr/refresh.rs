use anyhow::Result;
use crossterm::{execute, terminal::SetTitle};
use yazi_config::YAZI;
use yazi_fs::CWD;
use yazi_macro::{act, succ};
use yazi_parser::VoidOpt;
use yazi_shared::event::Data;
use yazi_term::tty::TTY;

use crate::{Actor, Ctx};

pub struct Refresh;

impl Actor for Refresh {
	type Options = VoidOpt;

	const NAME: &'static str = "refresh";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		if let (_, Some(s)) = (CWD.set(cx.cwd()), YAZI.mgr.title()) {
			execute!(TTY.writer(), SetTitle(s)).ok();
		}

		cx.tab_mut().apply_files_attrs();

		if let Some(p) = cx.parent() {
			cx.mgr.watcher.trigger_dirs(&[cx.current(), p]);
		} else {
			cx.mgr.watcher.trigger_dirs(&[cx.current()]);
		}

		act!(mgr:peek, cx, false)?;
		act!(mgr:watch, cx)?;
		act!(mgr:update_paged, cx)?;

		cx.tasks().prework_sorted(&cx.current().files);
		succ!();
	}
}
