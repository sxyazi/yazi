use std::{mem, time::{Duration, Instant}};

use anyhow::Result;
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::sync::oneshot;
use yazi_fs::File;
use yazi_macro::{act, succ};
use yazi_parser::mgr::DownloadOpt;
use yazi_shared::data::Data;
use yazi_vfs::VfsFile;

use crate::{Actor, Ctx};

pub struct Download;

impl Actor for Download {
	type Options = DownloadOpt;

	const NAME: &str = "download";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let mut wg1 = FuturesUnordered::new();
		for url in opt.urls {
			let (tx, rx) = oneshot::channel();
			cx.tasks.scheduler.file_download(url.to_owned(), Some(tx));
			wg1.push(async move { (rx.await == Ok(true), url) });
		}

		let scheduler = cx.tasks.scheduler.clone();
		tokio::spawn(async move {
			let mut wg2 = vec![];
			let mut files = Vec::with_capacity(wg1.len());
			let mut instant = Instant::now();
			while let Some((success, url)) = wg1.next().await {
				if !success {
					continue;
				}

				let Ok(f) = File::new(url).await else { continue };
				files.push(f);

				if instant.elapsed() >= Duration::from_secs(1) {
					wg2.push(scheduler.fetch_mimetype(mem::take(&mut files)));
					instant = Instant::now();
				}
			}

			if !files.is_empty() {
				wg2.push(scheduler.fetch_mimetype(files));
			}
			futures::future::join_all(wg2).await;
		});

		succ!();
	}
}
