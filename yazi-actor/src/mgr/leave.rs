use anyhow::Result;
use yazi_core::{Ctx, mgr::CdSource};
use yazi_macro::succ;
use yazi_parser::{VoidForm, spark::SparkKind};
use yazi_shared::{Source, data::Data, url::UrlLike};

use crate::{Actor, act};

pub struct Leave;

impl Actor for Leave {
	type Form = VoidForm;

	const NAME: &str = "leave";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let url =
			cx.hovered().and_then(|h| h.parent()).filter(|u| u != cx.cwd()).or_else(|| cx.cwd().parent());

		let Some(url) = url else { succ!() };
		let url = url.physical().to_owned();

		act!(mgr:cd, cx, (url, CdSource::Leave))
	}

	fn hook(cx: &Ctx, _: &Self::Form) -> Option<SparkKind> {
		match cx.source() {
			Source::Key => Some(SparkKind::KeyLeave),
			Source::Ind => Some(SparkKind::IndLeave),
			Source::Emit => Some(SparkKind::EmitLeave),
			Source::Relay => Some(SparkKind::RelayLeave),
			_ => None,
		}
	}
}
