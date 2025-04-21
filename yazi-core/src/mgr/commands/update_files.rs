use yazi_fs::FilesOp;
use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::{mgr::{LINKED, Mgr}, tab::Folder, tasks::Tasks};

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

		let revision = self.current().files.revision;
		let linked: Vec<_> = LINKED.read().from_dir(opt.op.cwd()).map(|u| opt.op.rebase(u)).collect();
		for op in [opt.op].into_iter().chain(linked) {
			self.yanked.apply_op(&op);
			self.update_tab(op, tasks);
		}

		render!(self.yanked.catchup_revision(false));
		self.active_mut().apply_files_attrs();

		if revision != self.current().files.revision {
			self.active_mut().hover(None);
			self.peek(false);
			self.watch(());
			self.update_paged((), tasks);
		}
	}

	fn update_tab(&mut self, op: FilesOp, tasks: &Tasks) {
		let url = op.cwd();
		self.active_mut().selected.apply_op(&op);

		if url == self.cwd() {
			self.update_current(op, tasks);
		} else if matches!(self.parent(), Some(p) if *url == p.url) {
			self.update_parent(op);
		} else if matches!(self.hovered(), Some(h) if *url == h.url) {
			self.update_hovered(op);
		} else {
			self.update_history(op);
		}
	}

	fn update_parent(&mut self, op: FilesOp) {
		let tab = self.active_mut();

		let urn = tab.current.url.urn();
		let leave = matches!(op, FilesOp::Deleting(_, ref urns) if urns.contains(urn));

		if let Some(f) = tab.parent.as_mut() {
			render!(f.update_pub(tab.id, op));
			render!(f.hover(urn));
		}

		if leave {
			tab.leave(());
		}
	}

	fn update_current(&mut self, op: FilesOp, tasks: &Tasks) {
		let calc = !matches!(op, FilesOp::Size(..) | FilesOp::Deleting(..));

		let id = self.active().id;
		if !self.current_mut().update_pub(id, op) {
			return;
		}

		if calc {
			tasks.prework_sorted(&self.current().files);
		}
	}

	fn update_hovered(&mut self, op: FilesOp) {
		let (id, url) = (self.active().id, op.cwd());
		let folder = self.active_mut().history.entry(url.clone()).or_insert_with(|| Folder::from(url));

		if folder.update_pub(id, op) {
			self.peek(true);
		}
	}

	fn update_history(&mut self, op: FilesOp) {
		let tab = &mut self.active_mut();
		let leave = tab.parent.as_ref().and_then(|f| f.url.parent_url().map(|p| (p, f.url.urn()))).is_some_and(
			|(p, n)| matches!(op, FilesOp::Deleting(ref parent, ref urns) if *parent == p && urns.contains(n)),
		);

		tab
			.history
			.entry(op.cwd().clone())
			.or_insert_with(|| Folder::from(op.cwd()))
			.update_pub(tab.id, op);

		if leave {
			tab.leave(());
		}
	}
}
