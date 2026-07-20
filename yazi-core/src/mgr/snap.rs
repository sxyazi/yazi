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

		// Fall back to the hovered item when nothing is explicitly selected, the
		// same convention `Tab::selected_or_hovered_urls()` used before #4108.
		let urls: Box<dyn Iterator<Item = &UrlBuf>> =
			match tab.checked_sub(1).and_then(|tab| self.tabs.get(tab)) {
				Some(s) if !s.selected.is_empty() => Box::new(s.selected.iter()),
				Some(s) => Box::new(s.hovered.iter()),
				None => Box::new(iter::empty()),
			};

		urls.skip(idx.unwrap_or(0)).take(if idx.is_some() { 1 } else { usize::MAX }).map(AsUrl::as_url)
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
