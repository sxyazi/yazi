use yazi_shared::{Id, Ids};

pub fn fetcher_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}

pub fn preloader_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
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
