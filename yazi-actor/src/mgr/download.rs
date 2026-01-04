use std::{mem, time::{Duration, Instant}};

use anyhow::Result;
use futures::{StreamExt, stream::FuturesUnordered};
use hashbrown::HashSet;
use yazi_fs::{File, FsScheme, provider::{Provider, local::Local}};
use yazi_macro::succ;
use yazi_parser::mgr::{DownloadOpt, OpenOpt};
use yazi_proxy::MgrProxy;
use yazi_shared::{data::Data, url::{UrlCow, UrlLike}};
use yazi_vfs::VfsFile;

use crate::{Actor, Ctx};

pub struct Download;

impl Actor for Download {
	type Options = DownloadOpt;

	const NAME: &str = "download";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let cwd = cx.cwd().clone();
		let scheduler = cx.tasks.scheduler.clone();

		tokio::spawn(async move {
			Self::prepare(&opt.urls).await;

			let mut wg1 = FuturesUnordered::new();
			for url in opt.urls {
				let done = scheduler.file_download(url.to_owned());
				wg1.push(async move { (done.future().await, url) });
			}

			let mut wg2 = vec![];
			let mut urls = Vec::with_capacity(wg1.len());
			let mut files = Vec::with_capacity(wg1.len());
			let mut instant = Instant::now();
			while let Some((success, url)) = wg1.next().await {
				if !success {
					continue;
				}

				let Ok(f) = File::new(&url).await else { continue };
				urls.push(url);
				files.push(f);

				if instant.elapsed() >= Duration::from_secs(1) {
					wg2.push(scheduler.fetch_mimetype(mem::take(&mut files)));
					instant = Instant::now();
				}
			}

			if !files.is_empty() {
				wg2.push(scheduler.fetch_mimetype(files));
			}
			if futures::future::join_all(wg2).await.into_iter().any(|b| !b) {
				return;
			}
			if opt.open && !urls.is_empty() {
				MgrProxy::open(OpenOpt {
					cwd:         Some(cwd.into()),
					targets:     urls,
					interactive: false,
					hovered:     false,
				});
			}
		});

		succ!();
	}
}

impl Download {
	async fn prepare(urls: &[UrlCow<'_>]) {
		let roots: HashSet<_> = urls.iter().filter_map(|u| u.scheme().cache()).collect();
		for mut root in roots {
			root.push("%lock");
			Local::regular(&root).create_dir_all().await.ok();
		}
	}
}
