use anyhow::Result;
use hashbrown::{HashMap, HashSet};
use mlua::Function;
use parking_lot::RwLock;
use yazi_boot::BOOT;
use yazi_fs::FolderStage;
use yazi_shared::{Id, RoCell, url::{Url, UrlBuf, UrlBufCov}};

use crate::{Client, ID, PEERS, ember::{BodyDuplicateItem, BodyMoveItem, Ember, EmberBulk, EmberHi}};

pub static LOCAL: RoCell<RwLock<HashMap<String, HashMap<String, Function>>>> = RoCell::new();

pub static REMOTE: RoCell<RwLock<HashMap<String, HashMap<String, Function>>>> = RoCell::new();

macro_rules! sub {
	($var:ident) => {
		|plugin: &str, kind: &str, f: Function| {
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

macro_rules! pub_after {
	(&impl $name:ident ($($param:ident: $param_ty:ty),*), ($($borrowed:expr),*), ($($owned:expr),*), $static:literal) => {
		paste::paste! {
			pub fn [<pub_after_ $name>]($($param: $param_ty),*) -> Result<()> {
				use crate::ember::[<Ember $name:camel>] as B;

				let n = if $static { concat!("@", stringify!($name)) } else { stringify!($name) };
				if BOOT.local_events.contains(n) {
					B::borrowed($($borrowed),*).with_receiver(*ID).flush()?;
				}
				if ($static && Self::any_remote_own(n)) || (!$static && PEERS.read().values().any(|p| p.able(n))) {
					Client::push(B::borrowed($($borrowed),*))?;
				}
				if LOCAL.read().contains_key(n) {
					Self::r#pub(B::owned($($owned),*))?;
				}
				Ok(())
			}
		}
	};
	($name:ident ($($param:ident: $param_ty:ty),*), ($($borrowed:expr),*)) => {
		pub_after!(&impl $name($($param: $param_ty),*), ($($borrowed),*), ($($borrowed),*), false);
	};
	($name:ident ($($param:ident: $param_ty:ty),*), ($($borrowed:expr),*), ($($owned:expr),*)) => {
		pub_after!(&impl $name($($param: $param_ty),*), ($($borrowed),*), ($($owned),*), false);
	};
	(@ $name:ident ($($param:ident: $param_ty:ty),*), ($($borrowed:expr),*)) => {
		pub_after!(&impl $name($($param: $param_ty),*), ($($borrowed),*), ($($borrowed),*), true);
	};
	(@ $name:ident ($($param:ident: $param_ty:ty),*), ($($borrowed:expr),*), ($($owned:expr),*)) => {
		pub_after!(&impl $name($($param: $param_ty),*), ($($borrowed),*), ($($owned),*), true);
	};
}

pub struct Pubsub;

impl Pubsub {
	pub fn sub(plugin: &str, kind: &str, f: Function) -> bool { sub!(LOCAL)(plugin, kind, f) }

	pub fn sub_remote(plugin: &str, kind: &str, f: Function) -> bool {
		sub!(REMOTE)(plugin, kind, f) && Self::pub_inner_hi()
	}

	pub fn unsub(plugin: &str, kind: &str) -> bool { unsub!(LOCAL)(plugin, kind) }

	pub fn unsub_remote(plugin: &str, kind: &str) -> bool {
		unsub!(REMOTE)(plugin, kind) && Self::pub_inner_hi()
	}

	pub fn r#pub(body: Ember<'static>) -> Result<()> { body.with_receiver(*ID).emit() }

	pub fn pub_to(receiver: Id, body: Ember<'static>) -> Result<()> {
		if receiver == *ID {
			return Self::r#pub(body);
		}

		let kind = body.kind();
		if receiver == 0 && Self::any_remote_own(kind) {
			Client::push(body)?;
		} else if PEERS.read().get(&receiver).is_some_and(|c| c.able(kind)) {
			Client::push(body.with_receiver(receiver))?;
		}
		Ok(())
	}

	pub fn pub_inner_hi() -> bool {
		let abilities = REMOTE.read().keys().cloned().collect();
		let abilities = BOOT.remote_events.union(&abilities).map(AsRef::as_ref);

		// FIXME: handle error
		Client::push(EmberHi::borrowed(abilities)).ok();
		true
	}

	pub fn pub_after_bulk<'a, I>(changes: I) -> Result<()>
	where
		I: Iterator<Item = (Url<'a>, Url<'a>)> + Clone,
	{
		if BOOT.local_events.contains("bulk") {
			EmberBulk::borrowed(changes.clone()).with_receiver(*ID).flush()?;
		}
		if PEERS.read().values().any(|p| p.able("bulk")) {
			Client::push(EmberBulk::borrowed(changes.clone()))?;
		}
		if LOCAL.read().contains_key("bulk") {
			Self::r#pub(EmberBulk::owned(changes))?;
		}
		Ok(())
	}

	fn any_remote_own(kind: &str) -> bool {
		REMOTE.read().contains_key(kind)  // Own remote abilities
			|| PEERS.read().values().any(|p| p.able(kind))  // Remote peers' abilities
			|| BOOT.remote_events.contains(kind) // Own abilities from the command-line argument
	}
}

impl Pubsub {
	pub_after!(tab(idx: Id), (idx));

	pub_after!(cd(tab: Id, url: &UrlBuf), (tab, url));

	pub_after!(load(tab: Id, url: &UrlBuf, stage: &FolderStage), (tab, url, stage));

	pub_after!(hover(tab: Id, url: Option<&UrlBuf>), (tab, url));

	pub_after!(rename(tab: Id, from: &UrlBuf, to: &UrlBuf), (tab, from, to));

	pub_after!(@yank(cut: bool, urls: &HashSet<UrlBufCov>), (cut, urls));

	pub_after!(duplicate(items: Vec<BodyDuplicateItem>), (&items), (items));

	pub_after!(move(items: Vec<BodyMoveItem>), (&items), (items));

	pub_after!(trash(urls: Vec<UrlBuf>), (&urls), (urls));

	pub_after!(delete(urls: Vec<UrlBuf>), (&urls), (urls));

	pub_after!(mount(), ());
}
