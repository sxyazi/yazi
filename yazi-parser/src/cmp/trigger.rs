use std::borrow::Cow;

use yazi_shared::{Id, event::{CmdCow, Data}};

pub struct TriggerOpt {
	pub word:   Cow<'static, str>,
	pub ticket: Id,
}

impl From<CmdCow> for TriggerOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			word:   c.take_first_str().unwrap_or_default(),
			ticket: c.get("ticket").and_then(Data::as_id).unwrap_or_default(),
		}
	}
}
