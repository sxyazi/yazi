use anyhow::bail;
use yazi_proxy::options::CmpItem;
use yazi_shared::{Id, event::CmdCow};

pub struct CompleteOpt {
	pub item:    CmpItem,
	pub _ticket: Id, // FIXME: not used
}

impl TryFrom<CmdCow> for CompleteOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(item) = c.take_any("item") else {
			bail!("Invalid 'item' in CompleteOpt");
		};

		Ok(Self { item, _ticket: c.id("ticket").unwrap_or_default() })
	}
}
