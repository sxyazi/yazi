use tokio::sync::mpsc;
use yazi_shared::{auth::AuthKind, url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike}};

use crate::{WATCHED, local::LINKED, r#virtual::VirtualReport};

#[derive(Clone)]
pub(crate) struct Reporter {
	pub(super) local_tx:   mpsc::UnboundedSender<UrlBuf>,
	pub(super) virtual_tx: mpsc::UnboundedSender<VirtualReport>,
}

impl Reporter {
	pub(crate) fn report<'a, I>(&self, urls: I)
	where
		I: IntoIterator,
		I::Item: Into<UrlCow<'a>>,
	{
		for url in urls.into_iter().map(Into::into) {
			match url.as_url().kind() {
				AuthKind::Regular | AuthKind::Search => self.report_local(url),
				AuthKind::Mount | AuthKind::Hub | AuthKind::Scope | AuthKind::Sftp => {
					self.report_virtual(url)
				}
			}
		}
	}

	fn report_local(&self, url: UrlCow) {
		let Some((parent, urn)) = url.pair() else { return };

		// FIXME: LINKED should return Url instead of Path
		let linked = LINKED.read();
		let linked = linked.from_dir(parent).map(Url::regular);

		let watched = WATCHED.read();
		for parent in [parent].into_iter().chain(linked) {
			if watched.contains_url(parent) {
				self.local_tx.send(url.to_owned()).ok();
				self.local_tx.send(parent.to_owned()).ok();
			}

			if urn.ext().is_some_and(|e| e == "%tmp") {
				continue;
			}

			// Virtual caches
			let Some(dir) = watched.find_by_cache(parent.loc()) else { continue };
			let Some(key) = url.name() else { continue };
			self.virtual_tx.send(VirtualReport::Cache(dir, key.to_owned())).ok();
		}
	}

	fn report_virtual(&self, url: UrlCow) {
		let Some(parent) = url.parent() else { return };
		if !WATCHED.read().contains_url(parent) {
			return;
		}

		self.virtual_tx.send(VirtualReport::Url(parent.to_owned())).ok();
		self.virtual_tx.send(VirtualReport::Url(url.into_owned())).ok();
	}
}
