use std::collections::{HashMap, HashSet};

use mlua::Function;
use parking_lot::RwLock;
use yazi_boot::BOOT;
use yazi_shared::{fs::Url, RoCell};

use crate::{body::{Body, BodyBulk, BodyCd, BodyDelete, BodyHi, BodyHover, BodyMove, BodyMoveItem, BodyRename, BodyTrash, BodyYank}, Client, ID, PEERS};

pub static LOCAL: RoCell<RwLock<HashMap<String, HashMap<String, Function<'static>>>>> =
	RoCell::new();

pub static REMOTE: RoCell<RwLock<HashMap<String, HashMap<String, Function<'static>>>>> =
	RoCell::new();

macro_rules! sub {
	($var:ident) => {
		|plugin: &str, kind: &str, f: Function<'static>| {
			let mut var = $var.write();
			let Some(map) = var.get_mut(kind) else {
				var.insert(kind.to_owned(), HashMap::from_iter([(plugin.to_owned(), f)]));
				return true;
			};

			if !map.contains_key(plugin) {
				map.insert(plugin.to_owned(), f);
				return true;
			}
			false
		}
	};
}

macro_rules! unsub {
	($var:ident) => {
		|plugin: &str, kind: &str| {
			let mut var = $var.write();
			let Some(map) = var.get_mut(kind) else { return false };

			if map.remove(plugin).is_none() {
				return false;
			}
			if map.is_empty() {
				var.remove(kind);
			}
			true
		}
	};
}

pub struct Pubsub;

impl Pubsub {
	pub fn sub(plugin: &str, kind: &str, f: Function<'static>) -> bool {
		sub!(LOCAL)(plugin, kind, f)
	}

	pub fn sub_remote(plugin: &str, kind: &str, f: Function<'static>) -> bool {
		sub!(REMOTE)(plugin, kind, f) && Self::pub_from_hi()
	}

	pub fn unsub(plugin: &str, kind: &str) -> bool { unsub!(LOCAL)(plugin, kind) }

	pub fn unsub_remote(plugin: &str, kind: &str) -> bool {
		unsub!(REMOTE)(plugin, kind) && Self::pub_from_hi()
	}

	pub fn pub_(body: Body<'static>) { body.with_receiver(*ID).emit(); }

	pub fn pub_to(receiver: u64, body: Body<'static>) {
		if receiver == *ID {
			return Self::pub_(body);
		}

		let (kind, peers) = (body.kind(), PEERS.read());
		if receiver == 0 && peers.values().any(|c| c.able(kind)) {
			Client::push(body);
		} else if peers.get(&receiver).is_some_and(|c| c.able(kind)) {
			Client::push(body.with_receiver(receiver));
		}
	}

	pub fn pub_static(severity: u16, body: Body) {
		if Self::own_static_ability(body.kind()) {
			Client::push(body.with_severity(severity));
		}
	}

	pub fn pub_from_hi() -> bool {
		let abilities = REMOTE.read().keys().cloned().collect();
		let abilities = BOOT.remote_events.union(&abilities).map(|s| s.as_str()).collect();

		Client::push(BodyHi::borrowed(abilities));
		true
	}

	pub fn pub_from_cd(tab: usize, url: &Url) {
		if LOCAL.read().contains_key("cd") {
			Self::pub_(BodyCd::dummy(tab));
		}
		if Self::own_static_ability("cd") {
			Client::push(BodyCd::borrowed(tab, url).with_severity(100));
		}
		if BOOT.local_events.contains("cd") {
			BodyCd::borrowed(tab, url).with_receiver(*ID).flush();
		}
	}

	pub fn pub_from_hover(tab: usize, url: Option<&Url>) {
		if LOCAL.read().contains_key("hover") {
			Self::pub_(BodyHover::dummy(tab));
		}
		if Self::own_static_ability("hover") {
			Client::push(BodyHover::borrowed(tab, url).with_severity(200));
		}
		if BOOT.local_events.contains("hover") {
			BodyHover::borrowed(tab, url).with_receiver(*ID).flush();
		}
	}

	pub fn pub_from_rename(tab: usize, from: &Url, to: &Url) {
		if LOCAL.read().contains_key("rename") {
			Self::pub_(BodyRename::dummy(tab, from, to));
		}
		if PEERS.read().values().any(|p| p.able("rename")) {
			Client::push(BodyRename::borrowed(tab, from, to));
		}
		if BOOT.local_events.contains("rename") {
			BodyRename::borrowed(tab, from, to).with_receiver(*ID).flush();
		}
	}

	pub fn pub_from_bulk(changes: HashMap<&Url, &Url>) {
		if LOCAL.read().contains_key("bulk") {
			Self::pub_(BodyBulk::owned(&changes));
		}
		if PEERS.read().values().any(|p| p.able("bulk")) {
			Client::push(BodyBulk::borrowed(&changes));
		}
		if BOOT.local_events.contains("bulk") {
			BodyBulk::borrowed(&changes).with_receiver(*ID).flush();
		}
	}

	pub fn pub_from_yank(cut: bool, urls: &HashSet<Url>) {
		if LOCAL.read().contains_key("yank") {
			Self::pub_(BodyYank::dummy());
		}
		if Self::own_static_ability("yank") {
			Client::push(BodyYank::borrowed(cut, urls).with_severity(300));
		}
		if BOOT.local_events.contains("yank") {
			BodyYank::borrowed(cut, urls).with_receiver(*ID).flush();
		}
	}

	pub(super) fn pub_from_move(items: Vec<BodyMoveItem>) {
		if PEERS.read().values().any(|p| p.able("move")) {
			Client::push(BodyMove::borrowed(&items));
		}
		if BOOT.local_events.contains("move") {
			BodyMove::borrowed(&items).with_receiver(*ID).flush();
		}
		if LOCAL.read().contains_key("move") {
			Self::pub_(BodyMove::owned(items));
		}
	}

	pub(super) fn pub_from_trash(urls: Vec<Url>) {
		if PEERS.read().values().any(|p| p.able("trash")) {
			Client::push(BodyTrash::borrowed(&urls));
		}
		if BOOT.local_events.contains("trash") {
			BodyTrash::borrowed(&urls).with_receiver(*ID).flush();
		}
		if LOCAL.read().contains_key("trash") {
			Self::pub_(BodyTrash::owned(urls));
		}
	}

	pub(super) fn pub_from_delete(urls: Vec<Url>) {
		if PEERS.read().values().any(|p| p.able("delete")) {
			Client::push(BodyDelete::borrowed(&urls));
		}
		if BOOT.local_events.contains("delete") {
			BodyDelete::borrowed(&urls).with_receiver(*ID).flush();
		}
		if LOCAL.read().contains_key("delete") {
			Self::pub_(BodyDelete::owned(urls));
		}
	}

	#[inline]
	fn own_static_ability(kind: &str) -> bool {
		REMOTE.read().contains_key(kind)  // Owned abilities
			|| PEERS.read().values().any(|p| p.able(kind))  // Remote peers' abilities
			|| BOOT.remote_events.contains(kind) // Owned abilities from the command-line argument
	}
}
