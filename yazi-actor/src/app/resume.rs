use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::app::ReflowForm;
use yazi_shared::data::Data;
use yazi_tui::Raterm;

use crate::{Actor, Ctx};

pub struct Resume;

impl Actor for Resume {
	type Form = ReflowForm;

	const NAME: &str = "resume";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		cx.active_mut().preview.reset();

		drop(cx.term.take());
		*cx.term = Some(Raterm::start()?);

		// While the app resumes, it's possible that the terminal size has changed.
		// We need to trigger a resize, and render the UI based on the resized area.
		act!(app:resize, cx, form.reflow)?;

		act!(app:title, cx).ok();
		succ!(render!());
	}
}
