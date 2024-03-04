use std::{borrow::Cow, mem};

use yazi_config::{keymap::ControlCow, which::SortBy, WHICH};
use yazi_shared::natsort;

#[derive(Clone, Copy, PartialEq)]
pub struct WhichSorter {
	pub by:        SortBy,
	pub sensitive: bool,
	pub reverse:   bool,
}

impl Default for WhichSorter {
	fn default() -> Self {
		Self {
			by:        WHICH.sort_by,
			sensitive: WHICH.sort_sensitive,
			reverse:   WHICH.sort_reverse,
		}
	}
}

impl WhichSorter {
	pub(super) fn sort(&self, items: &mut Vec<ControlCow>) {
		if self.by == SortBy::None || items.is_empty() {
			return;
		}

		let mut indices = Vec::with_capacity(items.len());
		let mut entities = Vec::with_capacity(items.len());
		for (i, ctrl) in items.iter().enumerate() {
			indices.push(i);
			entities.push(match self.by {
				SortBy::None => unreachable!(),
				SortBy::Key => Cow::Owned(ctrl.on()),
				SortBy::Desc => ctrl.desc_or_run(),
			});
		}

		indices.sort_unstable_by(|&a, &b| {
			let ordering = natsort(entities[a].as_bytes(), entities[b].as_bytes(), !self.sensitive);
			if self.reverse { ordering.reverse() } else { ordering }
		});

		*items = indices.into_iter().map(|i| mem::take(&mut items[i])).collect();
	}
}
