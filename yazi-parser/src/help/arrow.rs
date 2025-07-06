use yazi_shared::event::CmdCow;
use yazi_widgets::Step;

pub struct ArrowOpt {
	pub step: Step,
}

impl From<CmdCow> for ArrowOpt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(|d| d.try_into().ok()).unwrap_or_default() }
	}
}

impl From<isize> for ArrowOpt {
	fn from(n: isize) -> Self { Self { step: n.into() } }
}
