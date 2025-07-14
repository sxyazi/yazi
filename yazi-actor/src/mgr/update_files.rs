use anyhow::Result;
use yazi_core::{mgr::LINKED, tab::Folder};
use yazi_fs::FilesOp;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::UpdateFilesOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct UpdateFiles;

impl Actor for UpdateFiles {
	type Options = UpdateFilesOpt;

	const NAME: &'static str = "update_files";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let revision = cx.current().files.revision;
		let linked: Vec<_> = LINKED.read().from_dir(opt.op.cwd()).map(|u| opt.op.rebase(u)).collect();
		for op in [opt.op].into_iter().chain(linked) {
			cx.mgr.yanked.apply_op(&op);
			Self::update_tab(cx, op).ok();
		}

		render!(cx.mgr.yanked.catchup_revision(false));
		cx.tab_mut().apply_files_attrs();

		if revision != cx.current().files.revision {
			act!(mgr:hover, cx)?;
			act!(mgr:peek, cx, false)?;
			act!(mgr:watch, cx)?;
			act!(mgr:update_paged, cx)?;
		}
		succ!();
	}
}

impl UpdateFiles {
	fn update_tab(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let url = op.cwd();
		cx.tab_mut().selected.apply_op(&op);

		if url == cx.cwd() {
			Self::update_current(cx, op)
		} else if matches!(cx.parent(), Some(p) if *url == p.url) {
			Self::update_parent(cx, op)
		} else if matches!(cx.hovered(), Some(h) if *url == h.url) {
			Self::update_hovered(cx, op)
		} else {
			Self::update_history(cx, op)
		}
	}

	fn update_parent(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let tab = cx.tab_mut();

		let urn = tab.current.url.urn();
		let leave = matches!(op, FilesOp::Deleting(_, ref urns) if urns.contains(urn));

		if let Some(f) = tab.parent.as_mut() {
			render!(f.update_pub(tab.id, op));
			render!(f.hover(urn));
		}

		if leave {
			act!(mgr:leave, cx)?;
		}
		succ!();
	}

	fn update_current(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let calc = !matches!(op, FilesOp::Size(..) | FilesOp::Deleting(..));

		let id = cx.tab().id;
		if !cx.current_mut().update_pub(id, op) {
			succ!();
		}

		if calc {
			cx.tasks.prework_sorted(&cx.current().files);
		}
		succ!();
	}

	fn update_hovered(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let (id, url) = (cx.tab().id, op.cwd());
		let folder = cx.tab_mut().history.entry(url.clone()).or_insert_with(|| Folder::from(url));

		if folder.update_pub(id, op) {
			act!(mgr:peek, cx, true)?;
		}
		succ!();
	}

	fn update_history(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let tab = &mut cx.tab_mut();
		let leave = tab.parent.as_ref().and_then(|f| f.url.parent_url().map(|p| (p, f.url.urn()))).is_some_and(
			|(p, n)| matches!(op, FilesOp::Deleting(ref parent, ref urns) if *parent == p && urns.contains(n)),
		);

		tab
			.history
			.entry(op.cwd().clone())
			.or_insert_with(|| Folder::from(op.cwd()))
			.update_pub(tab.id, op);

		if leave {
			act!(mgr:leave, cx)?;
		}
		succ!();
	}
}
