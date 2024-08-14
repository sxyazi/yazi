use std::str::FromStr;

use anyhow::bail;
use serde::Deserialize;
use yazi_shared::fs::Cha;

#[derive(Default, Deserialize)]
#[serde(try_from = "String")]
pub enum Is {
	#[default]
	None,
	Hidden,
	Link,
	Orphan,
	Dummy,
	Block,
	Char,
	Fifo,
	Sock,
	Exec,
	Sticky,
}

impl FromStr for Is {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"hidden" => Self::Hidden,
			"link" => Self::Link,
			"orphan" => Self::Orphan,
			"dummy" => Self::Dummy,
			"block" => Self::Block,
			"char" => Self::Char,
			"fifo" => Self::Fifo,
			"sock" => Self::Sock,
			"exec" => Self::Exec,
			"sticky" => Self::Sticky,
			_ => bail!("invalid filetype: {s}"),
		})
	}
}

impl TryFrom<String> for Is {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::from_str(&s) }
}

impl Is {
	#[inline]
	pub fn check(&self, cha: &Cha) -> bool {
		match self {
			Self::None => true,
			Self::Hidden => cha.is_hidden(),
			Self::Link => cha.is_link(),
			Self::Orphan => cha.is_orphan(),
			Self::Dummy => cha.is_dummy(),
			Self::Block => cha.is_block(),
			Self::Char => cha.is_char(),
			Self::Fifo => cha.is_fifo(),
			Self::Sock => cha.is_sock(),
			Self::Exec => cha.is_exec(),
			Self::Sticky => cha.is_sticky(),
		}
	}
}
