use anyhow::Result;
use yazi_core::mgr::DisplaceOpt;
use yazi_macro::succ;
use yazi_parser::VoidForm;
use yazi_proxy::MgrProxy;
use yazi_shared::{data::Data, url::UrlLike};
use yazi_vfs::provider;

use crate::{Actor, Ctx};

pub struct Displace;

impl Actor for Displace {
	type Options = VoidForm;

	const NAME: &str = "displace";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		if cx.cwd().is_absolute() {
			succ!();
		}

		let tab = cx.tab().id;
		let from = cx.cwd().to_owned();
		tokio::spawn(async move {
			MgrProxy::displace_do(tab, DisplaceOpt {
				to: provider::canonicalize(&from).await.map_err(Into::into),
				from,
			});
		});

		succ!();
	}
}
