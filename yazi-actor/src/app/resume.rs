use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::app::ResumeOpt;
use yazi_shared::data::Data;
use yazi_term::Term;

use crate::{Actor, Ctx};

pub struct Resume;

impl Actor for Resume {
	type Options = ResumeOpt;

	const NAME: &str = "resume";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		cx.core.active_mut().preview.reset();
		*cx.term = Some(Term::start()?);

		// While the app resumes, it's possible that the terminal size has changed.
		// We need to trigger a resize, and render the UI based on the resized area.
		act!(app:resize, cx, opt.reflow)?;

		opt.tx.send((true, opt.token))?;

		succ!(render!());
	}
}
