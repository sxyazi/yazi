use serde::Deserialize;
use yazi_fs::stat::Stat;

#[derive(Clone, Copy, Default, Deserialize)]
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
	pub(crate) fn enabled(self) -> Option<Self> { (!matches!(self, Self::None)).then_some(self) }

	pub(crate) fn check(self, stat: &Stat) -> bool {
		match self {
			Self::None => true,
			Self::Hidden => stat.is_hidden(),
			Self::Link => stat.is_link(),
			Self::Orphan => stat.is_orphan(),
			Self::Dummy => stat.is_dummy(),
			Self::Block => stat.is_block(),
			Self::Char => stat.is_char(),
			Self::Fifo => stat.is_fifo(),
			Self::Sock => stat.is_sock(),
			Self::Exec => stat.is_exec(),
			Self::Sticky => stat.is_sticky(),
		}
	}
}
