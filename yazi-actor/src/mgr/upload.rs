use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::mgr::UploadForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Upload;

impl Actor for Upload {
	type Form = UploadForm;

	const NAME: &str = "upload";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		for url in form.urls {
			cx.tasks.scheduler.file_upload(url.into_owned());
		}
		succ!();
	}
}
