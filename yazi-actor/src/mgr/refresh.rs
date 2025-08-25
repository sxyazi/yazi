use anyhow::Result;
use crossterm::{execute, terminal::SetTitle};
use yazi_config::YAZI;
use yazi_core::tab::Folder;
use yazi_fs::{CWD, Files, FilesOp, cha::Cha};
use yazi_macro::{act, succ};
use yazi_parser::VoidOpt;
use yazi_shared::{event::Data, url::UrlBuf};
use yazi_term::tty::TTY;

use crate::{Actor, Ctx};

pub struct Refresh;

impl Actor for Refresh {
	type Options = VoidOpt;

	const NAME: &str = "refresh";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		if let (_, Some(s)) = (CWD.set(cx.cwd()), YAZI.mgr.title()) {
			execute!(TTY.writer(), SetTitle(s)).ok();
		}

		if let Some(p) = cx.parent() {
			Self::trigger_dirs(&[cx.current(), p]);
		} else {
			Self::trigger_dirs(&[cx.current()]);
		}

		act!(mgr:peek, cx)?;
		act!(mgr:watch, cx)?;
		act!(mgr:update_paged, cx)?;

		cx.tasks().prework_sorted(&cx.current().files);
		succ!();
	}
}

impl Refresh {
	// TODO: performance improvement
	pub fn trigger_dirs(folders: &[&Folder]) {
		async fn go(cwd: UrlBuf, cha: Cha) {
			let Some(cha) = Files::assert_stale(&cwd, cha).await else { return };

			match Files::from_dir_bulk(&cwd).await {
				Ok(files) => FilesOp::Full(cwd, files, cha).emit(),
				Err(e) => FilesOp::issue_error(&cwd, e.kind()).await,
			}
		}

		let futs: Vec<_> = folders
			.iter()
			.filter(|&f| f.url.is_internal())
			.map(|&f| go(f.url.to_owned(), f.cha))
			.collect();

		if !futs.is_empty() {
			tokio::spawn(futures::future::join_all(futs));
		}
	}
}
