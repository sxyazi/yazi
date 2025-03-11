use serde::Deserialize;
use yazi_fs::cha::Cha;

#[derive(Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
