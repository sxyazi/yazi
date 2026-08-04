use anyhow::Result;
use yazi_actor::Ctx;
use yazi_macro::{succ, writef};
use yazi_parser::{app::TitleForm, spark::SparkKind};
use yazi_shared::{Source, data::Data, url::UrlLike};
use yazi_tty::{TTY, sequence::{SetTitle, SetWorkingDir}};
use yazi_tui::RatermState;

use crate::Actor;

pub struct Title;

impl Actor for Title {
	type Form = TitleForm;

	const NAME: &str = "title";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let s = form.value.unwrap_or_else(|| format!("Yazi: {}", cx.tab().name()).into());
		writef!(TTY.writer(), "{}", SetTitle(&s))?;

		let cwd = cx.cwd().as_local().unwrap_or(std::path::Path::new("/"));
		writef!(TTY.writer(), "{}", SetWorkingDir(cwd))?;

		yazi_tui::STATE.set(RatermState { title: !s.is_empty(), ..yazi_tui::STATE.get() });
		succ!()
	}

	fn hook(cx: &Ctx, _form: &Self::Form) -> Option<SparkKind> {
		match cx.source() {
			Source::Ind => Some(SparkKind::IndAppTitle),
			_ => None,
		}
	}
}
