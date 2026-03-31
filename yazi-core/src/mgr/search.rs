use std::str::FromStr;

use anyhow::bail;
use serde::Deserialize;
use yazi_shared::{SStr, event::ActionCow, url::{UrlCow, UrlLike}};

#[derive(Clone, Debug)]
pub struct SearchOpt {
	pub via:      SearchVia,
	pub subject:  SStr,
	pub args:     Vec<String>,
	pub args_raw: SStr,
	pub r#in:     Option<UrlCow<'static>>,
}

impl TryFrom<ActionCow> for SearchOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let r#in = a.take::<UrlCow>("in").ok();
		if let Some(u) = &r#in
			&& (!u.is_absolute() || u.is_search())
		{
			bail!("invalid 'in' in SearchOpt");
		}

		let Ok(args) = yazi_shared::shell::unix::split(a.str("args"), false) else {
			bail!("invalid 'args' in SearchOpt");
		};

		Ok(Self {
			via: a.str("via").parse()?,
			subject: a.take_first().unwrap_or_default(),
			args: args.0,
			args_raw: a.take("args").unwrap_or_default(),
			r#in,
		})
	}
}

// Via
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SearchVia {
	Rg,
	Rga,
	Fd,
}

impl FromStr for SearchVia {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}

impl SearchVia {
	pub fn into_str(self) -> &'static str {
		match self {
			Self::Rg => "rg",
			Self::Rga => "rga",
			Self::Fd => "fd",
		}
	}
}
