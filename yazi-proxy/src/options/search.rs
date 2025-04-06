use std::borrow::Cow;

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

impl AsRef<str> for SearchOptVia {
	fn as_ref(&self) -> &str {
		match self {
			Self::Rg => "rg",
			Self::Rga => "rga",
			Self::Fd => "fd",
		}
	}
}
