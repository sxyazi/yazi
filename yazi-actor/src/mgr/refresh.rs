use anyhow::Result;
use crossterm::{execute, terminal::SetTitle};
use yazi_config::YAZI;
use yazi_core::tab::Folder;
use yazi_fs::{CWD, Files, FilesOp, cha::Cha};
use yazi_macro::{act, succ};
use yazi_parser::VoidOpt;
use yazi_proxy::MgrProxy;
use yazi_shared::{data::Data, url::{UrlBuf, UrlLike}};
use yazi_term::tty::TTY;
use yazi_vfs::{VfsFiles, VfsFilesOp};

use crate::{Actor, Ctx};

pub struct Refresh;

impl Actor for Refresh {
	type Options = VoidOpt;

	const NAME: &str = "refresh";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		if let (_, Some(s)) = (CWD.set(cx.cwd(), Self::cwd_changed), YAZI.mgr.title()) {
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
	fn cwd_changed() {
		if CWD.load().kind().is_virtual() {
			MgrProxy::watch();
		}
	}

	// TODO: performance improvement
	fn trigger_dirs(folders: &[&Folder]) {
		async fn go(dir: UrlBuf, cha: Cha) {
			let Some(cha) = Files::assert_stale(&dir, cha).await else { return };

			match Files::from_dir_bulk(&dir).await {
				Ok(files) => FilesOp::Full(dir, files, cha).emit(),
				Err(e) => FilesOp::issue_error(&dir, e).await,
			}
		}

		let futs: Vec<_> = folders
			.iter()
			.filter(|&f| f.url.is_absolute() && f.url.is_internal())
			.map(|&f| go(f.url.clone(), f.cha))
			.collect();

		if !futs.is_empty() {
			tokio::spawn(futures::future::join_all(futs));
		}
	}
}
