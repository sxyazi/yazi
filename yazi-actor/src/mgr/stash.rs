use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::{mgr::StashForm, spark::SparkKind};
use yazi_shared::{Source, data::Data, url::{AsUrl, UrlLike}};

use crate::{Actor, Ctx};

pub struct Stash;

impl Actor for Stash {
	type Form = StashForm;

	const NAME: &str = "stash";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		if form.target.is_absolute() && form.target.is_internal() {
			cx.tab_mut().backstack.push(form.target.as_url());
		}

		succ!()
	}

	fn hook(cx: &Ctx, _form: &Self::Form) -> Option<SparkKind> {
		match cx.source() {
			Source::Ind => Some(SparkKind::IndStash),
			Source::Relay => Some(SparkKind::RelayStash),
			_ => None,
		}
	}
}
