use anyhow::{Result, bail};
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
			if let Ok(to) = provider::absolute(&from).await
				&& to.is_owned()
			{
				MgrProxy::displace_do(tab, DisplaceDoOpt { to: to.into(), from });
			}
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
		if !opt.to.is_absolute() {
			bail!("Target URL must be absolute");
		}

		if cx.cwd() != opt.from {
			succ!()
		} else if let Some(hovered) = cx.hovered()
			&& let Ok(url) = opt.to.try_join(hovered.urn())
		{
			act!(mgr:reveal, cx, (url, CdSource::Displace))
		} else {
			act!(mgr:cd, cx, (opt.to, CdSource::Displace))
		}
	}
}
