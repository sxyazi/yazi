use std::{borrow::Cow, fmt::Display};

use yazi_shared::event::CmdCow;

pub struct SearchOpt {
	pub via:      SearchOptVia,
	pub subject:  Cow<'static, str>,
	pub args:     Vec<String>,
	pub args_raw: Cow<'static, str>,
}

impl TryFrom<CmdCow> for SearchOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		// TODO: remove this
		let (via, subject) = if let Some(s) = c.take_str("via") {
			(s.as_ref().into(), c.take_first_str().unwrap_or_default())
		} else {
			(c.take_first_str().unwrap_or_default().as_ref().into(), "".into())
		};

		Ok(Self {
			via,
			subject,
			// TODO: use second positional argument instead of `args` parameter
			args: yazi_shared::shell::split_unix(c.str("args").unwrap_or_default(), false)
				.map_err(|_| ())?
				.0,
			args_raw: c.take_str("args").unwrap_or_default(),
		})
	}
}

// Via
#[derive(PartialEq, Eq)]
pub enum SearchOptVia {
	// TODO: remove `None` in the future
	None,
	Rg,
	Fd,
	Rga,
}

impl From<&str> for SearchOptVia {
	fn from(value: &str) -> Self {
		match value {
			"rg" => Self::Rg,
			"fd" => Self::Fd,
			"rga" => Self::Rga,
			_ => Self::None,
		}
	}
}

impl Display for SearchOptVia {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::Rg => "rg",
			Self::Fd => "fd",
			Self::Rga => "rga",
			Self::None => "none",
		})
	}
}
