use anyhow::Result;
use yazi_dds::spark::SparkKind;
use yazi_macro::succ;
use yazi_parser::mgr::StashOpt;
use yazi_shared::{Source, data::Data, url::AsUrl};

use crate::{Actor, Ctx};

pub struct Stash;

impl Actor for Stash {
	type Options = StashOpt;

	const NAME: &str = "stash";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if opt.target.is_regular() {
			cx.tab_mut().backstack.push(opt.target.as_url());
		}

		succ!()
	}

	fn hook(cx: &Ctx, _opt: &Self::Options) -> Option<SparkKind> {
		match cx.source() {
			Source::Relay => Some(SparkKind::RelayStash),
			Source::Ind => Some(SparkKind::IndStash),
			_ => None,
		}
	}
}
