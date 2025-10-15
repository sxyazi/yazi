use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::UploadOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Upload;

impl Actor for Upload {
	type Options = UploadOpt;

	const NAME: &str = "upload";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		for url in opt.urls {
			cx.tasks.scheduler.file_upload(url.into_owned());
		}
		succ!();
	}
}
