use anyhow::Result;
use yazi_core::which::WhichSorter;
use yazi_macro::{render, succ};
use yazi_parser::{spark::SparkKind, which::ActivateForm};
use yazi_shared::{Source, data::Data};

use crate::{Actor, Ctx};

pub struct Activate;

impl Actor for Activate {
	type Form = ActivateForm;

	const NAME: &str = "activate";

	fn act(cx: &mut Ctx, Self::Form { mut opt }: Self::Form) -> Result<Data> {
		opt.cands.retain(|c| c.on.len() > opt.times);
		WhichSorter::default().sort(&mut opt.cands);

		if opt.cands.is_empty() {
			succ!();
		}

		let which = &mut cx.which;
		which.tx = opt.tx;
		which.times = opt.times;
		which.cands = opt.cands;

		which.active = true;
		which.silent = opt.silent;
		succ!(render!());
	}

	fn hook(cx: &Ctx, _form: &Self::Form) -> Option<SparkKind> {
		match cx.source() {
			Source::Unknown => Some(SparkKind::IndWhichActivate),
			_ => None,
		}
	}
}
