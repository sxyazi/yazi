use anyhow::Result;
use yazi_macro::act;
use yazi_parser::{app::DeprecateOpt, notify::{PushLevel, PushOpt}};
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Deprecate;

impl Actor for Deprecate {
	type Options = DeprecateOpt;

	const NAME: &str = "deprecate";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(notify:push, cx, PushOpt {
			title:   "Deprecated API".to_owned(),
			content: opt.content.into_owned(),
			level:   PushLevel::Warn,
			timeout: std::time::Duration::from_secs(20),
		})
	}
}
