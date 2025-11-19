use tokio::sync::mpsc;
use yazi_shared::{scheme::SchemeKind, url::{AsUrl, Url, UrlBuf, UrlCow, UrlLike}};

use crate::{WATCHED, local::LINKED};

#[derive(Clone)]
pub(crate) struct Reporter {
	pub(super) local_tx:  mpsc::UnboundedSender<UrlBuf>,
	pub(super) remote_tx: mpsc::UnboundedSender<UrlBuf>,
}

impl Reporter {
	pub(crate) fn report<'a, I>(&self, urls: I)
	where
		I: IntoIterator,
		I::Item: Into<UrlCow<'a>>,
	{
		for url in urls.into_iter().map(Into::into) {
			match url.as_url().kind() {
				SchemeKind::Regular | SchemeKind::Search => self.report_local(url),
				SchemeKind::Archive => {}
				SchemeKind::Sftp => self.report_remote(url),
			}
		}
	}

	fn report_local<'a>(&self, url: UrlCow<'a>) {
		let Some((parent, name)) = url.pair() else { return };

		// FIXME: LINKED should return Url instead of Path
		let linked = LINKED.read();
		let linked = linked.from_dir(parent).map(Url::regular);

		let watched = WATCHED.read();
		for parent in [parent].into_iter().chain(linked) {
			if watched.contains(parent) {
				self.local_tx.send(url.to_owned()).ok();
				self.local_tx.send(parent.to_owned()).ok();
			}
			if name.ext().is_some_and(|e| e == "%tmp") {
				continue;
			}
			// SFTP caches
			// todo!();
			// if let Some(dir) = watched.find_by_cache(&parent.loc()) {
			// 	self.remote_tx.send(dir.join(name)).ok();
			// 	self.remote_tx.send(dir.to_owned()).ok();
			// }
		}
	}

	fn report_remote<'a>(&self, url: UrlCow<'a>) {
		let Some(parent) = url.parent() else { return };
		if !WATCHED.read().contains(parent) {
			return;
		}

		self.remote_tx.send(parent.to_owned()).ok();
		self.remote_tx.send(url.into_owned()).ok();
	}
}
