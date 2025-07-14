use std::collections::HashMap;

use anyhow::Result;
use yazi_core::mgr::LINKED;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::UpdateMimesOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct UpdateMimes;

impl Actor for UpdateMimes {
	type Options = UpdateMimesOpt;

	const NAME: &'static str = "update_mimes";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let linked = LINKED.read();
		let updates = opt
			.updates
			.into_iter()
			.flat_map(|(key, value)| key.into_url().zip(value.into_string()))
			.filter(|(url, mime)| cx.mgr.mimetype.by_url(url) != Some(mime))
			.fold(HashMap::new(), |mut map, (u, m)| {
				for u in linked.from_file(&u) {
					map.insert(u, m.to_string());
				}
				map.insert(u, m.into_owned());
				map
			});

		drop(linked);
		if updates.is_empty() {
			succ!();
		}

		let affected: Vec<_> = cx
			.current()
			.paginate(cx.current().page)
			.iter()
			.filter(|&f| updates.contains_key(&f.url))
			.cloned()
			.collect();

		let repeek = cx.hovered().is_some_and(|f| updates.contains_key(&f.url));
		cx.mgr.mimetype.extend(updates);

		if repeek {
			act!(mgr:peek, cx, false)?;
		}
		cx.tasks.fetch_paged(&affected, &cx.mgr.mimetype);
		cx.tasks.preload_paged(&affected, &cx.mgr.mimetype);

		succ!(render!());
	}
}
