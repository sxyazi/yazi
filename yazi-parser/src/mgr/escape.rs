use bitflags::bitflags;
use yazi_shared::event::CmdCow;

bitflags! {
	pub struct EscapeOpt: u8 {
		const FIND   = 0b00001;
		const VISUAL = 0b00010;
		const FILTER = 0b00100;
		const SELECT = 0b01000;
		const SEARCH = 0b10000;
	}
}

impl From<CmdCow> for EscapeOpt {
	fn from(c: CmdCow) -> Self {
		c.args.iter().fold(EscapeOpt::empty(), |acc, (k, v)| {
			match (k.as_str().unwrap_or(""), v.as_bool().unwrap_or(false)) {
				("all", true) => Self::all(),
				("find", true) => acc | Self::FIND,
				("visual", true) => acc | Self::VISUAL,
				("filter", true) => acc | Self::FILTER,
				("select", true) => acc | Self::SELECT,
				("search", true) => acc | Self::SEARCH,
				_ => acc,
			}
		})
	}
}
