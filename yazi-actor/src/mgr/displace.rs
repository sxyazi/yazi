use std::io;

use anyhow::Result;
use yazi_core::mgr::DisplaceOpt;
use yazi_macro::succ;
use yazi_parser::VoidForm;
use yazi_proxy::MgrProxy;
use yazi_shared::{data::Data, url::UrlLike};
use yazi_vfs::engine;

use crate::{Actor, Ctx};

pub struct Displace;

impl Actor for Displace {
	type Form = VoidForm;

	const NAME: &str = "displace";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		if cx.cwd().is_regular() {
			succ!();
		}

		let tab = cx.tab().id;
		let from = cx.cwd().to_owned();
		tokio::spawn(async move {
			let to = match engine::reroute(&from).await {
				Ok(file) => Ok(file),
				Err(e) if e.kind() == io::ErrorKind::Unsupported => return,
				Err(e) => Err(e.into()),
			};

			MgrProxy::displace_do(tab, DisplaceOpt { to, from });
		});

		succ!();
	}
}
