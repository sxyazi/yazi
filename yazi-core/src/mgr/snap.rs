use std::iter;

use yazi_fs::Splatable;
use yazi_shared::url::{AsUrl, Url, UrlBuf};

use crate::{mgr::Mgr, tab::TabSnap};

pub struct MgrSnap {
	tab:    usize,
	tabs:   Vec<TabSnap>,
	yanked: Vec<UrlBuf>,
}

impl From<&Mgr> for MgrSnap {
	fn from(value: &Mgr) -> Self {
		Self {
			tab:    value.tabs.cursor,
			tabs:   value.tabs.iter().map(Into::into).collect(),
			yanked: value.yanked.urls().cloned().collect(),
		}
	}
}

impl Splatable for MgrSnap {
	fn tab(&self) -> usize { self.tab + 1 }

	fn selected(&self, tab: usize, mut idx: Option<usize>) -> impl Iterator<Item = Url<'_>> {
		idx = idx.and_then(|i| i.checked_sub(1));
		// Fall back to the hovered item when nothing is explicitly selected, same
		// as `Tab::selected_or_hovered_urls()` used for file operations elsewhere.
		let urls: Box<dyn Iterator<Item = &UrlBuf>> =
			match tab.checked_sub(1).and_then(|tab| self.tabs.get(tab)) {
				Some(s) if !s.selected.is_empty() => Box::new(s.selected.iter()),
				Some(s) => Box::new(s.hovered.iter()),
				None => Box::new(iter::empty()),
			};
		urls
			.skip(idx.unwrap_or(0))
			.take(if idx.is_some() { 1 } else { usize::MAX })
			.map(AsUrl::as_url)
	}

	fn hovered(&self, tab: usize) -> Option<Url<'_>> {
		tab
			.checked_sub(1)
			.and_then(|tab| self.tabs.get(tab))
			.and_then(|tab| tab.hovered.as_ref())
			.map(AsUrl::as_url)
	}

	fn yanked(&self, mut idx: Option<usize>) -> impl Iterator<Item = Url<'_>> {
		idx = idx.and_then(|i| i.checked_sub(1));
		self
			.yanked
			.iter()
			.skip(idx.unwrap_or(0))
			.take(if idx.is_some() { 1 } else { usize::MAX })
			.map(AsUrl::as_url)
	}
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use super::*;

	fn url(s: &str) -> UrlBuf { UrlBuf::from(PathBuf::from(s)) }

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
	fn selected_falls_back_to_hovered_when_nothing_selected() {
		let s = snap(&[], Some("hovered/file"));
		let urls: Vec<_> = s.selected(1, None).collect();
		assert_eq!(urls, [url("hovered/file")]);
	}

	#[test]
	fn selected_prefers_actual_selection_over_hovered() {
		let s = snap(&["a", "b"], Some("hovered/file"));
		let urls: Vec<_> = s.selected(1, None).collect();
		assert_eq!(urls, [url("a"), url("b")]);
	}

	#[test]
	fn selected_is_empty_when_nothing_selected_or_hovered() {
		let s = snap(&[], None);
		assert_eq!(s.selected(1, None).count(), 0);
	}
}
