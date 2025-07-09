use yazi_shared::{Id, SStr, event::{CmdCow, Data}};

pub struct TriggerOpt {
	pub word:   SStr,
	pub ticket: Option<Id>,
}

impl From<CmdCow> for TriggerOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			word:   c.take_first_str().unwrap_or_default(),
			ticket: c.get("ticket").and_then(Data::as_id),
		}
	}
}
