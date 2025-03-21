use std::{borrow::Cow, mem};

use yazi_config::{YAZI, keymap::ChordCow, which::SortBy};
use yazi_shared::{natsort, translit::Transliterator};

#[derive(Clone, Copy, PartialEq)]
pub struct WhichSorter {
	pub by:        SortBy,
	pub sensitive: bool,
	pub reverse:   bool,
	pub translit:  bool,
}

impl Default for WhichSorter {
	fn default() -> Self {
		Self {
			by:        YAZI.which.sort_by,
			sensitive: YAZI.which.sort_sensitive,
			reverse:   YAZI.which.sort_reverse,
			translit:  YAZI.which.sort_translit,
		}
	}
}

impl WhichSorter {
	pub(super) fn sort(&self, items: &mut Vec<ChordCow>) {
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
			let ordering = if !self.translit {
				natsort(entities[a].as_bytes(), entities[b].as_bytes(), !self.sensitive)
			} else {
				natsort(
					entities[a].as_bytes().transliterate().as_bytes(),
					entities[b].as_bytes().transliterate().as_bytes(),
					!self.sensitive,
				)
			};

			if self.reverse { ordering.reverse() } else { ordering }
		});

		*items = indices.into_iter().map(|i| mem::take(&mut items[i])).collect();
	}
}
