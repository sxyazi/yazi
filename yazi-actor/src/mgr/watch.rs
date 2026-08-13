use std::iter;

use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::{mgr::WatchForm, spark::SparkKind};
use yazi_shared::{Source, data::Data};

use crate::{Actor, Ctx};

pub struct Watch;

impl Actor for Watch {
	type Form = WatchForm;

	const NAME: &str = "watch";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		if !form.files.is_empty() {
			succ!(cx.core.mgr.watcher.watch(form.files));
		}

		let tab = cx.core.mgr.tabs.active();
		let it = iter::once(&tab.current.file)
			.chain(tab.hovered_folder().map(|h| &h.file).or(tab.hovered().filter(|f| f.is_dir())))
			.chain(tab.parent.as_ref().map(|p| &p.file));

		succ!(cx.core.mgr.watcher.watch(it));
	}

	fn hook(cx: &Ctx, _: &Self::Form) -> Option<SparkKind> {
		(cx.source() == Source::Ind).then_some(SparkKind::IndWatch)
	}
}
