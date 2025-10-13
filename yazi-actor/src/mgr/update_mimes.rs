use anyhow::Result;
use hashbrown::HashMap;
use yazi_macro::{act, render, succ};
use yazi_parser::mgr::UpdateMimesOpt;
use yazi_shared::{data::Data, pool::InternStr, url::{AsUrl, UrlCov}};
use yazi_watcher::local::LINKED;

use crate::{Actor, Ctx};

pub struct UpdateMimes;

impl Actor for UpdateMimes {
	type Options = UpdateMimesOpt;

	const NAME: &str = "update_mimes";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let linked = LINKED.read();
		let updates = opt
			.updates
			.into_iter()
			.flat_map(|(key, value)| key.into_url().zip(value.into_string()))
			.filter(|(url, mime)| cx.mgr.mimetype.get(url) != Some(mime))
			.fold(HashMap::new(), |mut map, (u, m)| {
				for u in linked.from_file(u.as_url()) {
					map.insert(u.into(), m.intern());
				}
				map.insert(u.into(), m.intern());
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
			.filter(|&f| updates.contains_key(&UrlCov::new(&f.url)))
			.cloned()
			.collect();

		let repeek = cx.hovered().is_some_and(|f| updates.contains_key(&UrlCov::new(&f.url)));
		cx.mgr.mimetype.extend(updates);

		if repeek {
			act!(mgr:peek, cx)?;
		}
		cx.tasks.fetch_paged(&affected, &cx.mgr.mimetype);
		cx.tasks.preload_paged(&affected, &cx.mgr.mimetype);

		succ!(render!());
	}
}
