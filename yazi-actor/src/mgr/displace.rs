use anyhow::{Result, bail};
use yazi_fs::FilesOp;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, mgr::{CdSource, DisplaceDoOpt}};
use yazi_proxy::MgrProxy;
use yazi_shared::{data::Data, url::UrlLike};
use yazi_vfs::provider;

use crate::{Actor, Ctx};

pub struct Displace;

impl Actor for Displace {
	type Options = VoidOpt;

	const NAME: &str = "displace";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		if cx.cwd().is_absolute() {
			succ!();
		}

		let tab = cx.tab().id;
		let from = cx.cwd().to_owned();
		tokio::spawn(async move {
			MgrProxy::displace_do(tab, DisplaceDoOpt {
				to: provider::absolute(&from).await.map(|u| u.into_owned()),
				from,
			});
		});

		succ!();
	}
}

// --- Do
pub struct DisplaceDo;

impl Actor for DisplaceDo {
	type Options = DisplaceDoOpt;

	const NAME: &str = "displace_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if cx.cwd() != opt.from {
			succ!()
		}

		let to = match opt.to {
			Ok(url) => url,
			Err(e) => return act!(mgr:update_files, cx, FilesOp::IOErr(opt.from, e.into())),
		};

		if !to.is_absolute() {
			bail!("Target URL must be absolute");
		} else if let Some(hovered) = cx.hovered()
			&& let Ok(url) = to.try_join(hovered.urn())
		{
			act!(mgr:reveal, cx, (url, CdSource::Displace))
		} else {
			act!(mgr:cd, cx, (to, CdSource::Displace))
		}
	}
}
