use std::borrow::Cow;

use yazi_fs::Folder;
use yazi_proxy::ManagerProxy;
use yazi_shared::{event::Cmd, fs::FilesOp, render};

use crate::{manager::{Manager, LINKED}, tab::Tab, tasks::Tasks};

pub struct Opt {
	op: FilesOp,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self { op: c.take_any("op").ok_or(())? })
	}
}

impl Manager {
	pub fn update_files(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		let mut ops = vec![opt.op];
		for u in LINKED.read().from_dir(ops[0].url()) {
			ops.push(ops[0].chroot(u));
		}

		for op in ops {
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
		let url = op.url();
		tab.selected.apply_op(&op);

		if url == tab.cwd() {
			Self::update_current(tab, op, tasks);
		} else if matches!(&tab.parent, Some(p) if url == &*p.loc) {
			Self::update_parent(tab, op);
		} else if matches!(tab.current.hovered(), Some(h) if url == h.url()) {
			Self::update_hovered(tab, op);
		} else {
			Self::update_history(tab, op);
		}
	}

	fn update_parent(tab: &mut Tab, op: Cow<FilesOp>) {
		let cwd = tab.cwd().clone();
		let leave = matches!(*op, FilesOp::Deleting(_, ref urls) if urls.contains(&cwd));

		if let Some(f) = tab.parent.as_mut() {
			render!(f.update(op.into_owned()));
			render!(f.hover(&cwd));
		}

		if leave {
			tab.leave(());
		}
	}

	fn update_current(tab: &mut Tab, op: Cow<FilesOp>, tasks: &Tasks) {
		let hovered = tab.current.hovered().filter(|_| tab.current.tracing).map(|h| h.url_owned());
		let calc = !matches!(*op, FilesOp::Size(..) | FilesOp::Deleting(..));

		let foreign = matches!(op, Cow::Borrowed(_));
		if !tab.current.update(op.into_owned()) {
			return;
		}

		tab.current.repos(hovered);
		if foreign {
			return;
		}

		ManagerProxy::hover(None, tab.idx); // Re-hover in next loop
		ManagerProxy::update_paged(); // Update for paged files in next loop
		if calc {
			tasks.prework_sorted(&tab.current.files);
		}
	}

	fn update_hovered(tab: &mut Tab, op: Cow<FilesOp>) {
		let url = op.url();
		let folder = tab.history.entry(url.clone()).or_insert_with(|| Folder::from(url));

		let foreign = matches!(op, Cow::Borrowed(_));
		if !folder.update(op.into_owned()) {
			return;
		}

		if !foreign {
			ManagerProxy::peek(true);
		}
	}

	fn update_history(tab: &mut Tab, op: Cow<FilesOp>) {
		let leave = tab.parent.as_ref().and_then(|f| f.loc.parent_url().map(|p| (&f.loc, p))).is_some_and(
			|(p, pp)| matches!(*op, FilesOp::Deleting(ref parent, ref urls) if *parent == pp && urls.contains(p)),
		);

		let folder = tab.history.entry(op.url().clone()).or_insert_with(|| Folder::from(op.url()));
		let hovered = folder.hovered().filter(|_| folder.tracing).map(|h| h.url_owned());
		if folder.update(op.into_owned()) {
			folder.repos(hovered);
		}

		if leave {
			tab.leave(());
		}
	}
}
