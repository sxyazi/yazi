use anyhow::Result;
use crossterm::{execute, terminal::SetTitle};
use yazi_actor::Ctx;
use yazi_macro::succ;
use yazi_parser::{app::TitleForm, spark::SparkKind};
use yazi_shared::{Source, data::Data};
use yazi_term::TermState;
use yazi_tty::TTY;

use crate::Actor;

pub struct Title;

impl Actor for Title {
	type Form = TitleForm;

	const NAME: &str = "title";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let s = form.value.unwrap_or_else(|| format!("Yazi: {}", cx.tab().name()).into());
		execute!(TTY.writer(), SetTitle(&s))?;

		yazi_term::STATE.set(TermState { title: !s.is_empty(), ..yazi_term::STATE.get() });
		succ!()
	}

	fn hook(cx: &Ctx, _form: &Self::Form) -> Option<SparkKind> {
		match cx.source() {
			Source::Ind => Some(SparkKind::IndAppTitle),
			_ => None,
		}
	}
}
