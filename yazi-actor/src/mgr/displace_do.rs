use anyhow::{Result, bail};
use yazi_fs::FilesOp;
use yazi_macro::{act, succ};
use yazi_parser::mgr::{CdSource, DisplaceDoOpt};
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

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
