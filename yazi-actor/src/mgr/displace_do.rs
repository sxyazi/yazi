use anyhow::Result;
use yazi_core::mgr::CdSource;
use yazi_fs::FilesOp;
use yazi_macro::{act, succ};
use yazi_parser::mgr::DisplaceDoForm;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct DisplaceDo;

impl Actor for DisplaceDo {
	type Form = DisplaceDoForm;

	const NAME: &str = "displace_do";

	fn act(cx: &mut Ctx, Self::Form { opt }: Self::Form) -> Result<Data> {
		if cx.cwd() != opt.from {
			succ!()
		}

		let file = match opt.to {
			Ok(file) => file,
			Err(e) => return act!(mgr:update_files, cx, FilesOp::IOErr(opt.from, e)),
		};

		if file.is_dir() {
			cx.tab_mut().backstack.replace(&file.url);
		} else if let Some((trail, _)) = file.url.pair() {
			cx.tab_mut().backstack.replace(trail);
		}

		if file.is_file() {
			act!(mgr:reveal, cx, (file.url, CdSource::Displace))
		} else if let Some(hovered) = cx.hovered()
			&& let Ok(url) = file.url.try_join(hovered.urn())
		{
			act!(mgr:reveal, cx, (url, CdSource::Displace))
		} else {
			act!(mgr:cd, cx, (file.url, CdSource::Displace))
		}
	}
}
