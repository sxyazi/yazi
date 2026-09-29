use anyhow::Result;
use yazi_core::mgr::CdSource;
use yazi_macro::succ;
use yazi_parser::VoidForm;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx, act};

pub struct Leave;

impl Actor for Leave {
	type Form = VoidForm;

	const NAME: &str = "leave";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let url =
			cx.hovered().and_then(|h| h.parent()).filter(|u| u != cx.cwd()).or_else(|| cx.cwd().parent());

		let Some(url) = url else {
			Self::pick_drive();
			succ!()
		};
		let url = url.physical().to_owned();

		act!(mgr:cd, cx, (url, CdSource::Leave))
	}
}

impl Leave {
	/// On Windows the filesystem has no single root: leaving a drive root
	/// (e.g. `C:\`) has nowhere to go. Show a picker listing all mounted
	/// drives instead of doing nothing (issue #3240).
	#[cfg(windows)]
	fn pick_drive() {
		use yazi_config::YAZI;
		use yazi_proxy::{MgrProxy, PickProxy};

		tokio::spawn(async move {
			// Enumerating drives can block on stalled network drives
			let drives = tokio::task::spawn_blocking(yazi_fs::mounts::drives).await.unwrap_or_default();
			if drives.is_empty() {
				return;
			}

			let items: Vec<_> = drives
				.iter()
				.map(|d| {
					let root = d.src.to_string_lossy();
					match &d.label {
						Some(l) => format!("{root}  {}", l.to_string_lossy()),
						None => root.into_owned(),
					}
				})
				.collect();

			let mut cfg = YAZI.pick.open(items);
			cfg.title = "Drives".into();
			if let Some(i) = PickProxy::show(cfg).await
				&& let Some(dist) = drives.get(i).and_then(|d| d.dist.clone())
			{
				MgrProxy::cd(dist, CdSource::Leave);
			}
		});
	}

	#[cfg(not(windows))]
	fn pick_drive() {}
}
