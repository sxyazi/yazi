use yazi_config::{keymap::ControlCow, which::SortBy};
use yazi_shared::natsort;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct WhichSorter {
	pub by:        SortBy,
	pub sensitive: bool,
	pub reverse:   bool,
}

impl WhichSorter {
	pub(super) fn sort(&self, items: &mut Vec<ControlCow>) -> bool {
		if items.is_empty() {
			return false;
		}

		let by_alphabetical = |a: &str, b: &str| {
			let ordering = natsort(a.as_bytes(), b.as_bytes(), !self.sensitive);
			if self.reverse { ordering.reverse() } else { ordering }
		};

		match self.by {
			SortBy::None => return false,
			SortBy::Key => items.sort_unstable_by(|a, b| {
				let a = a.on.iter().map(|c| c.to_string()).collect::<String>();
				let b = b.on.iter().map(|c| c.to_string()).collect::<String>();
				by_alphabetical(&a, &b)
			}),
			SortBy::Desc => items.sort_unstable_by(|a, b| {
				// what if description isn't present (need to check if it's mandatory or not)
				// in case if it is not present, should I just panic ?
				by_alphabetical(a.desc.as_ref().unwrap(), b.desc.as_ref().unwrap())
			}),
		}

		true
	}
}
