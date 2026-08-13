use yazi_shared::id::{Id, Ids};

pub fn fetcher_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub fn fetcher_rev() -> u16 {
	static IDS: Ids = Ids::new();
	((IDS.next().get() - 1) % u16::MAX as u64 + 1) as u16
}

pub fn preloader_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub fn preloader_rev() -> u16 {
	static IDS: Ids = Ids::new();
	((IDS.next().get() - 1) % u16::MAX as u64 + 1) as u16
}

pub fn previewer_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub fn spotter_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub fn open_rule_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub fn opener_rule_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}
