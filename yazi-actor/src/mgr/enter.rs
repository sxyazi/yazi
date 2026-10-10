use anyhow::Result;
use yazi_core::{Ctx, mgr::CdSource};
use yazi_macro::succ;
use yazi_parser::{VoidForm, spark::SparkKind};
use yazi_shared::{Source, data::Data, url::UrlLike};

use crate::{Actor, act};

pub struct Enter;

impl Actor for Enter {
	type Form = VoidForm;

	const NAME: &str = "enter";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let Some(h) = cx.hovered().filter(|h| h.is_dir()) else { succ!() };

		let url = h.physical().to_owned();

		act!(mgr:cd, cx, (url, CdSource::Enter))
	}

	fn hook(cx: &Ctx, _: &Self::Form) -> Option<SparkKind> {
		match cx.source() {
			Source::Key => Some(SparkKind::KeyEnter),
			Source::Ind => Some(SparkKind::IndEnter),
			Source::Emit => Some(SparkKind::EmitEnter),
			Source::Relay => Some(SparkKind::RelayEnter),
			_ => None,
		}
	}
}
