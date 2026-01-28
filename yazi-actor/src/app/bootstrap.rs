use anyhow::Result;
use yazi_actor::Ctx;
use yazi_boot::BOOT;
use yazi_macro::{act, succ};
use yazi_parser::{VoidOpt, mgr::CdSource};
use yazi_shared::{data::Data, strand::StrandLike, url::UrlLike};

use crate::Actor;

pub struct Bootstrap;

impl Actor for Bootstrap {
	type Options = VoidOpt;

	const NAME: &str = "bootstrap";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		for (i, file) in BOOT.files.iter().enumerate() {
			let tabs = &mut cx.core.mgr.tabs;
			if tabs.len() <= i {
				tabs.push(Default::default());
			}

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
