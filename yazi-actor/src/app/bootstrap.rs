use anyhow::Result;
use yazi_actor::Ctx;
use yazi_boot::BOOT;
use yazi_core::mgr::CdSource;
use yazi_macro::{act, succ};
use yazi_parser::VoidForm;
use yazi_shared::{data::Data, strand::StrandLike, url::UrlLike};

use crate::Actor;

pub struct Bootstrap;

impl Actor for Bootstrap {
	type Form = VoidForm;

	const NAME: &str = "bootstrap";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		cx.mgr.tabs.resize_with(BOOT.files.len(), Default::default);

		for (i, file) in BOOT.files.iter().enumerate().rev() {
			cx.tab = i;
			if file.is_empty() {
				act!(mgr:cd, cx, (BOOT.cwds[i].clone(), CdSource::Tab))?;
			} else if let Ok(u) = BOOT.cwds[i].try_join(file) {
				act!(mgr:reveal, cx, (u, CdSource::Tab))?;
			}
		}

		succ!();
	}
}
