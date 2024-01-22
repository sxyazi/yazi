use std::borrow::Cow;

use yazi_shared::{event::Exec, fs::FilesOp, render};

use crate::{folder::Folder, manager::Manager, tab::Tab, tasks::Tasks};

pub struct Opt {
	op: FilesOp,
}

impl TryFrom<Exec> for Opt {
	type Error = ();

	fn try_from(mut e: Exec) -> Result<Self, Self::Error> {
		Ok(Self { op: e.take_data().ok_or(())? })
	}
}

impl Manager {
	fn update_tab(tab: &mut Tab, op: Cow<FilesOp>, tasks: &Tasks) {
		let url = op.url();
		if tab.current.cwd == *url {
			Self::update_current(tab, op, tasks);
		} else if matches!(&tab.parent, Some(p) if p.cwd == *url) {
			Self::update_parent(tab, op);
		} else if matches!(tab.current.hovered(), Some(h) if h.url == *url) {
			Self::update_hovered(tab, op);
		} else {
			Self::update_history(tab, op);
		}
	}

	fn update_parent(tab: &mut Tab, op: Cow<FilesOp>) {
		let cwd = tab.current.cwd.clone();
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
		let hovered = tab.current.hovered().filter(|_| tab.current.tracing).map(|h| h.url());
		let calc = !matches!(*op, FilesOp::Size(..) | FilesOp::Deleting(..));

		let foreign = matches!(op, Cow::Borrowed(_));
		if !tab.current.update(op.into_owned()) {
			return;
		}

		tab.current.repos(hovered);
		if foreign {
			return;
		}

		Self::_hover(None); // Re-hover in next loop
		Self::_update_paged(); // Update for paged files in next loop
		if calc {
			tasks.preload_sorted(&tab.current.files);
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
			Self::_peek(true);
		}
	}

	fn update_history(tab: &mut Tab, op: Cow<FilesOp>) {
		let leave = tab.parent.as_ref().and_then(|f| f.cwd.parent_url().map(|p| (&f.cwd, p))).is_some_and(
			|(p, pp)| matches!(*op, FilesOp::Deleting(ref parent, ref urls) if *parent == pp && urls.contains(p)),
		);

		if let Some(f) = tab.history.get_mut(op.url()) {
			let hovered = f.hovered().filter(|_| f.tracing).map(|h| h.url());
			_ = f.update(op.into_owned()) && f.repos(hovered);
		}

		if leave {
			tab.leave(());
		}
	}

	pub fn update_files(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		let mut ops = vec![opt.op];
		for u in self.watcher.linked.read().from_dir(ops[0].url()) {
			ops.push(ops[0].chroot(u));
		}

		for op in ops {
			let idx = self.tabs.idx;
			for (_, tab) in self.tabs.iter_mut().enumerate().filter(|(i, _)| *i != idx) {
				Self::update_tab(tab, Cow::Borrowed(&op), tasks);
			}

			Self::update_tab(self.active_mut(), Cow::Owned(op), tasks);
		}

		self.active_mut().apply_files_attrs();
	}
}
