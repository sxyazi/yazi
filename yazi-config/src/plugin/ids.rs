use yazi_shared::id::{Id, Ids};

pub(crate) fn fetcher_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub(crate) fn fetcher_rev() -> u16 {
	static IDS: Ids = Ids::new();
	((IDS.next().get() - 1) % u16::MAX as u64 + 1) as u16
}

pub(crate) fn preloader_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub(crate) fn preloader_rev() -> u16 {
	static IDS: Ids = Ids::new();
	((IDS.next().get() - 1) % u16::MAX as u64 + 1) as u16
}

pub(crate) fn previewer_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub(crate) fn spotter_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub(crate) fn open_rule_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub(crate) fn opener_rule_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}
