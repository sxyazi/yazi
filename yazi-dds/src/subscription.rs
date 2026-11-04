use std::{borrow::Borrow, hash::{Hash, Hasher}};

use mlua::{Function, MetaMethod, UserData, UserDataMethods, Value};
use yazi_shared::id::{Id, Ids};
use yazi_shim::CompactString;

use crate::{LOCAL, Pubsub, REMOTE};

static IDS: Ids = Ids::new();

#[derive(Clone)]
pub struct Subscription {
	pub(super) id:   Id,
	pub(super) kind: CompactString,
	pub plugin:      CompactString,
	pub callback:    Function,
	remote:          bool,
}

// Subscription access, cloning, and dropping must stay on the owning Lua thread;
// background threads only inspect event names in the registry.
unsafe impl Send for Subscription {}
unsafe impl Sync for Subscription {}

impl Borrow<str> for Subscription {
	fn borrow(&self) -> &str { &self.plugin }
}

impl Hash for Subscription {
	fn hash<H: Hasher>(&self, state: &mut H) { self.plugin.hash(state); }
}

impl PartialEq for Subscription {
	fn eq(&self, other: &Self) -> bool { self.plugin == other.plugin }
}

impl Eq for Subscription {}

impl Subscription {
	pub(super) fn new(kind: &str, plugin: &str, callback: Function) -> Self {
		Self { id: IDS.next(), kind: kind.into(), plugin: plugin.into(), callback, remote: false }
	}

	pub(super) fn with_remote(mut self) -> Self {
		self.remote = true;
		self
	}
}

impl UserData for Subscription {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Close, |_, me, _: Value| {
			let removed = if me.remote { REMOTE.write() } else { LOCAL.write() }.remove_exact(me);
			if removed && me.remote {
				Pubsub::pub_inner_hi();
			}
			Ok(())
		});
	}
}
