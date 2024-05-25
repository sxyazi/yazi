use std::str::FromStr;

use anyhow::bail;
use serde::Deserialize;
use yazi_shared::fs::Cha;

#[derive(Default, Deserialize)]
#[serde(try_from = "String")]
pub enum Is {
	#[default]
	None,
	Block,
	Char,
	Exec,
	Fifo,
	Link,
	Orphan,
	Sock,
	Sticky,
}

impl FromStr for Is {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"block" => Self::Block,
			"char" => Self::Char,
			"exec" => Self::Exec,
			"fifo" => Self::Fifo,
			"link" => Self::Link,
			"orphan" => Self::Orphan,
			"sock" => Self::Sock,
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
			Self::Block => cha.is_block(),
			Self::Char => cha.is_char(),
			Self::Exec => cha.is_exec(),
			Self::Fifo => cha.is_fifo(),
			Self::Link => cha.is_link(),
			Self::Orphan => cha.is_orphan(),
			Self::Sock => cha.is_sock(),
			Self::Sticky => cha.is_sticky(),
		}
	}
}
