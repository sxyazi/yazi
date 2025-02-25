use std::borrow::Cow;

use yazi_fs::FilesOp;
use yazi_macro::render;
use yazi_proxy::MgrProxy;
use yazi_shared::event::CmdCow;

use crate::{mgr::{LINKED, Mgr}, tab::{Folder, Tab}, tasks::Tasks};

pub struct Opt {
	op: FilesOp,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { op: c.take_any("op").ok_or(())? })
	}
}

impl Mgr {
	pub fn update_files(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		let linked: Vec<_> = LINKED.read().from_dir(opt.op.cwd()).map(|u| opt.op.rebase(u)).collect();
		for op in [opt.op].into_iter().chain(linked) {
			let idx = self.tabs.cursor;
			self.yanked.apply_op(&op);

			for (_, tab) in self.tabs.iter_mut().enumerate().filter(|(i, _)| *i != idx) {
				Self::update_tab(tab, Cow::Borrowed(&op), tasks);
			}
			Self::update_tab(self.active_mut(), Cow::Owned(op), tasks);
		}

		render!(self.yanked.catchup_revision(false));
		self.active_mut().apply_files_attrs();
	}

	fn update_tab(tab: &mut Tab, op: Cow<FilesOp>, tasks: &Tasks) {
		let url = op.cwd();
		tab.selected.apply_op(&op);

		if url == tab.cwd() {
			Self::update_current(tab, op, tasks);
		} else if matches!(&tab.parent, Some(p) if *url == p.url) {
			Self::update_parent(tab, op);
		} else if matches!(tab.hovered(), Some(h) if *url == h.url) {
			Self::update_hovered(tab, op);
		} else {
			Self::update_history(tab, op);
		}
	}

	fn update_parent(tab: &mut Tab, op: Cow<FilesOp>) {
		let urn = tab.cwd().urn_owned();
		let leave = matches!(*op, FilesOp::Deleting(_, ref urns) if urns.contains(&urn));

		if let Some(f) = tab.parent.as_mut() {
			render!(f.update_pub(tab.id, op.into_owned()));
			render!(f.hover(urn.as_urn()));
		}

		if leave {
			tab.leave(());
		}
	}

	fn update_current(tab: &mut Tab, op: Cow<FilesOp>, tasks: &Tasks) {
		let calc = !matches!(*op, FilesOp::Size(..) | FilesOp::Deleting(..));
		let foreign = matches!(op, Cow::Borrowed(_));

		if !tab.current.update_pub(tab.id, op.into_owned()) {
			return;
		} else if foreign {
			return;
		}

		MgrProxy::hover(None, tab.id); // Re-hover in next loop
		MgrProxy::update_paged(); // Update for paged files in next loop
		if calc {
			tasks.prework_sorted(&tab.current.files);
		}
	}

	fn update_hovered(tab: &mut Tab, op: Cow<FilesOp>) {
		let url = op.cwd();
		let folder = tab.history.entry(url.clone()).or_insert_with(|| Folder::from(url));

		let foreign = matches!(op, Cow::Borrowed(_));
		if !folder.update_pub(tab.id, op.into_owned()) {
			return;
		}

		if !foreign {
			MgrProxy::peek(true);
		}
	}

	fn update_history(tab: &mut Tab, op: Cow<FilesOp>) {
		let leave = tab.parent.as_ref().and_then(|f| f.url.parent_url().map(|p| (p, f.url.urn()))).is_some_and(
			|(p, n)| matches!(*op, FilesOp::Deleting(ref parent, ref urns) if *parent == p && urns.contains(n)),
		);

		tab
			.history
			.entry(op.cwd().clone())
			.or_insert_with(|| Folder::from(op.cwd()))
			.update_pub(tab.id, op.into_owned());

		if leave {
			tab.leave(());
		}
	}
}
