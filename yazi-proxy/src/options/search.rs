use anyhow::bail;
use yazi_shared::{SStr, event::CmdCow};

pub struct SearchOpt {
	pub via:      SearchOptVia,
	pub subject:  SStr,
	pub args:     Vec<String>,
	pub args_raw: SStr,
}

impl TryFrom<CmdCow> for SearchOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		// TODO: remove this
		let (via, subject) = if let Some(s) = c.take_str("via") {
			(s.as_ref().into(), c.take_first_str().unwrap_or_default())
		} else {
			(c.take_first_str().unwrap_or_default().as_ref().into(), "".into())
		};

		let Ok(args) = yazi_shared::shell::split_unix(c.str("args").unwrap_or_default(), false) else {
			bail!("Invalid 'args' argument in SearchOpt");
		};

		Ok(Self {
			via,
			subject,
			// TODO: use second positional argument instead of `args` parameter
			args: args.0,
			args_raw: c.take_str("args").unwrap_or_default(),
		})
	}
}

// Via
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchOptVia {
	Rg,
	Rga,
	Fd,
}

impl From<&str> for SearchOptVia {
	fn from(value: &str) -> Self {
		match value {
			"rg" => Self::Rg,
			"rga" => Self::Rga,
			_ => Self::Fd,
		}
	}
}

impl SearchOptVia {
	pub fn into_str(self) -> &'static str {
		match self {
			Self::Rg => "rg",
			Self::Rga => "rga",
			Self::Fd => "fd",
		}
	}
}
