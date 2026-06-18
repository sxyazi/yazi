use yazi_shared::id::{Id, Ids};

pub fn chord_id() -> Id {
	static IDS: Ids = Ids::new();
	IDS.next()
}
