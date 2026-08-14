use std::iter;

use anyhow::Result;
use yazi_core::{Invalidator, Reconciler};
use yazi_fs::FilesOp;
use yazi_macro::{act, render, succ};
use yazi_parser::{mgr::UpdateFilesForm, spark::SparkKind};
use yazi_shared::{Source, data::Data, url::UrlLike};
use yazi_watcher::local::LINKED;

use crate::{Actor, Ctx};

pub struct UpdateFiles;

impl Actor for UpdateFiles {
	type Form = UpdateFilesForm;

	const NAME: &str = "update_files";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let linked: Vec<_> = LINKED.read().from_dir(form.op.cwd()).map(|u| form.op.chdir(u)).collect();
		let ops: Vec<_> = iter::once(form.op).chain(linked).collect();

		for op in &ops {
			Invalidator::new(&mut cx.mgr).apply(op);
			Reconciler::new(cx.tab, &mut cx.mgr).apply(op);
		}
		render!(cx.mgr.yanked.catchup_revision(false));

		let tabs = cx.tabs().indices_or_active(form.tabs);
		let Some((&last, tabs)) = tabs.split_last() else { succ!() };

		for &tab in tabs {
			cx.with(tab, |cx| Self::update_tab(cx, ops.iter().cloned()))?;
		}
		cx.with(last, |cx| Self::update_tab(cx, ops))
	}

	fn hook(cx: &Ctx, _: &Self::Form) -> Option<SparkKind> {
		(cx.source() == Source::Relay).then_some(SparkKind::RelayUpdateFiles)
	}
}

impl UpdateFiles {
	fn update_tab(cx: &mut Ctx, ops: impl IntoIterator<Item = FilesOp>) -> Result<Data> {
		let revision = cx.current().entries.revision;

		for op in ops {
			Self::update_pane(cx, op).ok();
		}

		act!(mgr:hidden, cx).ok();
		act!(mgr:sort, cx).ok();
		if revision != cx.current().entries.revision {
			act!(mgr:hover, cx)?;
			act!(mgr:peek, cx)?;
			act!(mgr:watch, cx).ok();
			act!(mgr:update_paged, cx)?;
		}
		succ!();
	}

	fn update_pane(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let url = op.cwd();

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

		let key = tab.current.url.key();
		let leave = matches!(op, FilesOp::Deleting(_, ref keys) if keys.contains(&key));

		if let Some(f) = tab.parent.as_mut() {
			render!(f.update_pub(tab.id, op));
			render!(f.hover(key));
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
			cx.tasks.prework_sorted(&cx.current().entries);
		}
		succ!();
	}

	fn update_hovered(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let (id, url) = (cx.tab().id, op.cwd());
		let (folder, _) = cx.tab_mut().history.ensure(url);

		if folder.update_pub(id, op) {
			act!(mgr:peek, cx, true)?;
		}
		succ!();
	}

	fn update_history(cx: &mut Ctx, op: FilesOp) -> Result<Data> {
		let tab = cx.tab_mut();
		let leave = tab.parent.as_ref().and_then(|f| f.url.pair()).is_some_and(
			|(t, key)| matches!(&op, FilesOp::Deleting(trail, keys) if trail == t && keys.contains(&key)),
		);

		let (folder, evicted) = tab.history.ensure(op.cwd());
		folder.update_pub(tab.id, op);

		if let Some(evicted) = evicted.filter(|f| tab.hovered_url() == Some(&f.url)) {
			tab.history.insert(evicted);
		}

		if leave {
			act!(mgr:leave, cx)?;
		}

		succ!();
	}
}
