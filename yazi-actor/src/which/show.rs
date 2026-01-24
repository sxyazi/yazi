use anyhow::Result;
use yazi_core::which::WhichSorter;
use yazi_dds::spark::SparkKind;
use yazi_macro::{render, succ};
use yazi_parser::which::ShowOpt;
use yazi_shared::{Source, data::Data};

use crate::{Actor, Ctx};

pub struct Show;

impl Actor for Show {
	type Options = ShowOpt;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, mut opt: Self::Options) -> Result<Data> {
		opt.cands.retain(|c| c.on.len() > opt.times);
		WhichSorter::default().sort(&mut opt.cands);

		if opt.cands.is_empty() {
			succ!();
		}

		let which = &mut cx.which;
		which.times = opt.times;
		which.cands = opt.cands;

		which.visible = true;
		which.silent = opt.silent;
		succ!(render!());
	}

	fn hook(cx: &Ctx, _opt: &Self::Options) -> Option<SparkKind> {
		match cx.source() {
			Source::Unknown => Some(SparkKind::IndWhichShow),
			_ => None,
		}
	}
}
