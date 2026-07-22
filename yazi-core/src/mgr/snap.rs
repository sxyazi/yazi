use yazi_fs::{Splatable, file::File};

use crate::{mgr::Mgr, tab::TabSnap};

pub struct MgrSnap {
	tab:    usize,
	tabs:   Vec<TabSnap>,
	yanked: Vec<File>,
}

impl From<&Mgr> for MgrSnap {
	fn from(value: &Mgr) -> Self {
		Self {
			tab:    value.tabs.cursor,
			tabs:   value.tabs.iter().map(Into::into).collect(),
			yanked: value.yanked.files().cloned().collect(),
		}
	}
}

impl Splatable for MgrSnap {
	fn tab(&self) -> usize { self.tab + 1 }

	fn selected(&self, tab: usize, mut idx: Option<usize>) -> impl Iterator<Item = &File> {
		idx = idx.and_then(|i| i.checked_sub(1));

		// Fall back to the hovered item when nothing is explicitly selected, the
		// same convention `Tab::selected_or_hovered_urls()` used before #4108.
		let urls: &[UrlBuf] = match tab.checked_sub(1).and_then(|tab| self.tabs.get(tab)) {
			Some(s) if !s.selected.is_empty() => &s.selected,
			Some(s) => s.hovered.as_slice(),
			None => &[],
		};

		urls
			.iter()
			.skip(idx.unwrap_or(0))
			.take(if idx.is_some() { 1 } else { usize::MAX })
	}

	fn hovered(&self, tab: usize) -> Option<&File> {
		tab.checked_sub(1).and_then(|tab| self.tabs.get(tab)).and_then(|tab| tab.hovered.as_ref())
	}

	fn yanked(&self, mut idx: Option<usize>) -> impl Iterator<Item = &File> {
		idx = idx.and_then(|i| i.checked_sub(1));
		self.yanked.iter().skip(idx.unwrap_or(0)).take(if idx.is_some() { 1 } else { usize::MAX })
	}
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;

	fn url(s: &str) -> UrlBuf { UrlBuf::from(Path::new(s)) }

	fn snap(selected: &[&str], hovered: Option<&str>) -> MgrSnap {
		MgrSnap {
			tab:    0,
			tabs:   vec![TabSnap {
				hovered:  hovered.map(url),
				selected: selected.iter().map(|s| url(s)).collect(),
			}],
			yanked: vec![],
		}
	}

	#[test]
	fn selected_falls_back_to_hovered_when_empty() {
		let s = snap(&[], Some("hovered"));

		assert_eq!(s.selected(1, None).collect::<Vec<_>>(), [url("hovered").as_url()]);
	}

	#[test]
	fn selected_prefers_explicit_selection_over_hovered() {
		let s = snap(&["a", "b"], Some("hovered"));

		assert_eq!(s.selected(1, None).collect::<Vec<_>>(), [url("a").as_url(), url("b").as_url()]);
	}

	#[test]
	fn selected_empty_when_nothing_selected_or_hovered() {
		let s = snap(&[], None);

		assert_eq!(s.selected(1, None).count(), 0);
	}
}
