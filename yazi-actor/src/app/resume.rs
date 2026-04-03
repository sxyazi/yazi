use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::app::ResumeForm;
use yazi_shared::data::Data;
use yazi_term::Term;

use crate::{Actor, Ctx};

pub struct Resume;

impl Actor for Resume {
	type Form = ResumeForm;

	const NAME: &str = "resume";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		cx.active_mut().preview.reset();
		*cx.term = Some(Term::start()?);

		// While the app resumes, it's possible that the terminal size has changed.
		// We need to trigger a resize, and render the UI based on the resized area.
		act!(app:resize, cx, form.reflow)?;

		form.tx.send((true, form.replier))?;

		act!(app:title, cx).ok();
		succ!(render!());
	}
}
