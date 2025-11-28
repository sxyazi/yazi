use std::borrow::Cow;

use percent_encoding::percent_decode;
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

	fn report_local(&self, url: UrlCow) {
		let Some((parent, urn)) = url.pair() else { return };

		// FIXME: LINKED should return Url instead of Path
		let linked = LINKED.read();
		let linked = linked.from_dir(parent).map(Url::regular);

		let watched = WATCHED.read();
		for parent in [parent].into_iter().chain(linked) {
			if watched.contains(parent) {
				self.local_tx.send(url.to_owned()).ok();
				self.local_tx.send(parent.to_owned()).ok();
			}

			if urn.ext().is_some_and(|e| e == "%tmp") {
				continue;
			}

			// Virtual caches
			let Some(dir) = watched.find_by_cache(parent.loc()) else { continue };
			let Some(name) = url.name() else { continue };
			if let Ok(u) = dir.try_join(Cow::from(percent_decode(name.encoded_bytes()))) {
				self.remote_tx.send(u).ok();
			}
			self.remote_tx.send(dir).ok();
		}
	}

	fn report_remote(&self, url: UrlCow) {
		let Some(parent) = url.parent() else { return };
		if !WATCHED.read().contains(parent) {
			return;
		}

		self.remote_tx.send(parent.to_owned()).ok();
		self.remote_tx.send(url.into_owned()).ok();
	}
}
