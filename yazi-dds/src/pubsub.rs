use std::collections::{HashMap, HashSet};

use mlua::Function;
use parking_lot::RwLock;
use yazi_boot::BOOT;
use yazi_shared::{fs::Url, RoCell};

use crate::{body::{Body, BodyCd, BodyHi, BodyHover, BodyRename, BodyTabs, BodyYank}, Client, Payload, ID, PEERS};

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

	pub fn pub_(body: Body<'static>) { body.upgrade().with_receiver(*ID).flush(false).emit(); }

	pub fn pub_to(receiver: u64, body: Body<'static>) {
		if receiver == *ID {
			return Self::pub_(body);
		}

		let (kind, peers) = (body.kind(), PEERS.read());
		if receiver == 0 && peers.values().any(|c| c.able(kind)) {
			Client::push(body.upgrade());
		} else if peers.get(&receiver).is_some_and(|c| c.able(kind)) {
			Client::push(body.upgrade().with_receiver(receiver));
		}
	}

	pub fn pub_static(severity: u8, body: Body) {
		let (kind, peers) = (body.kind(), PEERS.read());
		if peers.values().any(|c| c.able(kind)) {
			Client::push(body.upgrade().with_severity(severity));
		}
	}

	pub fn pub_from_hi() -> bool {
		let abilities = REMOTE.read().keys().cloned().collect();
		let abilities = BOOT.remote_events.union(&abilities).collect();

		Client::push(BodyHi::borrowed(abilities).upgrade());
		true
	}

	pub fn pub_from_tabs(tab: usize, urls: &[&Url]) {
		if LOCAL.read().contains_key("tabs") {
			Self::pub_(BodyTabs::dummy(tab));
		}
		if PEERS.read().values().any(|p| p.able("tabs")) {
			Client::push(BodyTabs::borrowed(tab, urls).upgrade());
		}
		if BOOT.local_events.contains("tabs") {
			BodyTabs::borrowed(tab, urls).upgrade().with_receiver(*ID).flush(true);
		}
	}

	pub fn pub_from_cd(tab: usize, url: &Url) {
		if LOCAL.read().contains_key("cd") {
			Self::pub_(BodyCd::dummy(tab));
		}
		if PEERS.read().values().any(|p| p.able("cd")) {
			Client::push(BodyCd::borrowed(tab, url).upgrade());
		}
		if BOOT.local_events.contains("cd") {
			BodyCd::borrowed(tab, url).upgrade().with_receiver(*ID).flush(true);
		}
	}

	pub fn pub_from_hover(tab: usize, url: Option<&Url>) {
		if LOCAL.read().contains_key("hover") {
			Self::pub_(BodyHover::dummy(tab));
		}
		if PEERS.read().values().any(|p| p.able("hover")) {
			Client::push(BodyHover::borrowed(tab, url).upgrade());
		}
		if BOOT.local_events.contains("hover") {
			BodyHover::borrowed(tab, url).upgrade().with_receiver(*ID).flush(true);
		}
	}

	pub fn pub_from_rename(tab: usize, from: &Url, to: &Url) {
		if LOCAL.read().contains_key("rename") {
			Self::pub_(BodyRename::dummy(tab, from, to));
		}
		if PEERS.read().values().any(|p| p.able("rename")) {
			Client::push(BodyRename::borrowed(tab, from, to).upgrade());
		}
		if BOOT.local_events.contains("rename") {
			BodyRename::borrowed(tab, from, to).upgrade().with_receiver(*ID).flush(true);
		}
	}

	pub fn pub_from_yank(cut: bool, urls: &HashSet<Url>) {
		if LOCAL.read().contains_key("yank") {
			Self::pub_(BodyYank::dummy(cut));
		}
		if PEERS.read().values().any(|p| p.able("yank")) {
			Client::push(BodyYank::borrowed(cut, urls).upgrade());
		}
		if BOOT.local_events.contains("yank") {
			BodyYank::borrowed(cut, urls).upgrade().with_receiver(*ID).flush(true);
		}
	}
}
