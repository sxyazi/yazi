use anyhow::Result;
use yazi_core::{Ctx, mgr::CdSource};
use yazi_fs::op::FilesOp;
use yazi_macro::succ;
use yazi_parser::mgr::DisplaceDoForm;
use yazi_shared::{data::Data, path::PathLike, url::UrlLike};

use crate::{Actor, act};

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
			Err(e) => return act!(mgr:update_files, cx, FilesOp::Fail(opt.from, e)),
		};

		// Replace the alias in history to avoid rerouting it again
		if file.is_dir() {
			cx.tab_mut().backstack.replace(&file.url);
		} else if let Some((trail, _)) = file.pair() {
			cx.tab_mut().backstack.replace(trail);
		}

		// Reveal files in their parent
		if file.is_file() {
			return act!(mgr:reveal, cx, (file.url, CdSource::Displace));
		}

		// Enter the resolved directory then restore its hover by key
		let trace = cx.current().trace.as_ref().map(|p| p.to_kind(file.key().kind()));
		act!(mgr:cd, cx, (file.url, CdSource::Displace))?;

		if let Some(Ok(trace)) = trace {
			cx.current_mut().trace = Some(trace);
			act!(mgr:hover, cx)?;
			act!(mgr:peek, cx)?;
			act!(mgr:watch, cx).ok();
		}
		succ!();
	}
}
