use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::UploadForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Upload;

impl Actor for Upload {
	type Options = UploadForm;

	const NAME: &str = "upload";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		for url in opt.urls {
			cx.tasks.scheduler.file_upload(url.into_owned());
		}
		succ!();
	}
}
