use std::fmt::Display;

use yazi_shared::event::Cmd;

pub struct SearchOpt {
	pub via:      SearchOptVia,
	pub subject:  String,
	pub args:     Vec<String>,
	pub args_raw: String,
}

impl TryFrom<Cmd> for SearchOpt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self {
			// TODO: remove `c.take_first_str()` in the future
			via:      c.take_str("via").or_else(|| c.take_first_str()).unwrap_or_default().into(),
			subject:  c.take_first_str().unwrap_or_default(),
			args:     shell_words::split(c.str("args").unwrap_or_default()).map_err(|_| ())?,
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
}

impl From<String> for SearchOptVia {
	fn from(value: String) -> Self {
		match value.as_str() {
			"rg" => Self::Rg,
			"fd" => Self::Fd,
			_ => Self::None,
		}
	}
}

impl Display for SearchOptVia {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::Rg => "rg",
			Self::Fd => "fd",
			Self::None => "none",
		})
	}
}
