use std::{io, time::{Duration, SystemTime}};

use hashbrown::{HashMap, HashSet};
use notify::Result;
use tokio::{pin, sync::mpsc::UnboundedReceiver};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_fs::FilesOp;
use yazi_shared::{strand::StrandBuf, url::{UrlBuf, UrlLike}};
use yazi_vfs::{Stamp, engine};

use crate::{MgrProxy, WATCHER, Watchee};

pub(crate) struct Virtual;

#[derive(Hash, PartialEq, Eq)]
pub(crate) enum VirtualReport {
	Url(UrlBuf),
	Cache(UrlBuf, StrandBuf),
}

impl Virtual {
	pub(crate) fn serve(rx: UnboundedReceiver<VirtualReport>) -> Self {
		tokio::spawn(Self::changed(rx));

		Self
	}

	pub(crate) fn watch(&mut self, _watchee: &mut Watchee) -> Result<()> { Ok(()) }

	pub(crate) fn unwatch(&mut self, _watchee: &Watchee) -> Result<()> { Ok(()) }

	async fn changed(rx: UnboundedReceiver<VirtualReport>) {
		let rx = UnboundedReceiverStream::new(rx).chunks_timeout(1000, Duration::from_millis(250));
		pin!(rx);

		while let Some(chunk) = rx.next().await {
			let reports: HashSet<_> = chunk.into_iter().collect();

			let mut urls = HashMap::with_capacity(reports.len());
			for report in reports {
				match report {
					VirtualReport::Url(url) => _ = urls.entry(url).or_insert(false),
					VirtualReport::Cache(dir, key) if let Ok(file) = Stamp::resolve(&dir, &key).await => {
						urls.insert(file, true);
						urls.entry(dir).or_insert(false);
					}
					VirtualReport::Cache(..) => {}
				}
			}

			let _permit = WATCHER.acquire().await.unwrap();

			let mut ops = Vec::with_capacity(urls.len());
			let mut ups = Vec::with_capacity(urls.len());

			for (url, upload) in urls {
				let Some((parent, key)) = url.pair2() else { continue };

				let mut file = match engine::file(&url).await {
					Ok(file) => file,
					Err(e) if e.kind() == io::ErrorKind::NotFound => {
						ops.push(FilesOp::Deleting(parent.into(), [key.into()].into()));
						continue;
					}
					Err(e) => {
						tracing::debug!("Failed to update {url:?}: {e:?}");
						continue;
					}
				};

				if upload && file.is_file() {
					file.cha.ctime = Some(SystemTime::now());
					ops.push(FilesOp::Upserting(parent.into(), [(key.into(), file)].into()));
					ups.push(url);
					continue;
				}

				ops.push(FilesOp::Upserting(parent.into(), [(key.into(), file)].into()));
			}

			FilesOp::mutate(ops);
			MgrProxy::upload(ups);
		}
	}
}
