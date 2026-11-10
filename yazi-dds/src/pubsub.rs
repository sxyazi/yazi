use anyhow::Result;
use mlua::Function;
use parking_lot::RwLock;
use yazi_boot::BOOT;
use yazi_fs::{FolderStage, file::FileCov, op::FilesOp};
use yazi_shared::{id::Id, url::{Url, UrlBuf}};
use yazi_shim::indexmap::IndexSet;

use crate::{Client, PEERS, Subscription, Subscriptions, ember::{Ember, EmberDuplicateItem, EmberHi, EmberKind, EmberMoveItem}};

pub(crate) static LOCAL: RwLock<Subscriptions> = RwLock::new(Subscriptions::new());

pub(crate) static REMOTE: RwLock<Subscriptions> = RwLock::new(Subscriptions::new());

macro_rules! pub_after {
	(&impl $name:ident $(<$lt:lifetime>)? ($($param:ident: $param_ty:ty),*), ($($borrowed:expr),*), ($($owned:expr),*), $static:literal) => {
		paste::paste! {
			pub fn [<pub_after_ $name>]$(<$lt>)?($($param: $param_ty),*) -> Result<()> {
				use crate::ember::[<Ember $name:camel>] as B;

				let n: &'static str = EmberKind::[<$name:camel>].into();
				if BOOT.local_events.contains(n) {
					B::borrowed($($borrowed),*).with_receiver(BOOT.id).flush()?;
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
	($name:ident $(<$lt:lifetime>)? ($($param:ident: $param_ty:ty),*), ($($borrowed:expr),*)) => {
		pub_after!(&impl $name$(<$lt>)?($($param: $param_ty),*), ($($borrowed),*), ($($borrowed),*), false);
	};
	($name:ident $(<$lt:lifetime>)? ($($param:ident: $param_ty:ty),*), ($($borrowed:expr),*), ($($owned:expr),*)) => {
		pub_after!(&impl $name$(<$lt>)?($($param: $param_ty),*), ($($borrowed),*), ($($owned),*), false);
	};
	(@ $name:ident $(<$lt:lifetime>)? ($($param:ident: $param_ty:ty),*), ($($borrowed:expr),*)) => {
		pub_after!(&impl $name$(<$lt>)?($($param: $param_ty),*), ($($borrowed),*), ($($borrowed),*), true);
	};
	(@ $name:ident $(<$lt:lifetime>)? ($($param:ident: $param_ty:ty),*), ($($borrowed:expr),*), ($($owned:expr),*)) => {
		pub_after!(&impl $name$(<$lt>)?($($param: $param_ty),*), ($($borrowed),*), ($($owned),*), true);
	};
}

pub struct Pubsub;

impl Pubsub {
	pub fn sub(kind: &str, plugin: &str, f: Function) -> Option<Subscription> {
		LOCAL.write().insert(Subscription::new(kind, plugin, f))
	}

	pub fn sub_remote(kind: &str, plugin: &str, f: Function) -> Option<Subscription> {
		let subscription = REMOTE.write().insert(Subscription::new(kind, plugin, f).with_remote());
		if subscription.is_some() {
			Self::pub_inner_hi();
		}
		subscription
	}

	pub fn unsub(kind: &str, plugin: &str) -> bool { LOCAL.write().remove(kind, plugin) }

	pub fn unsub_remote(kind: &str, plugin: &str) -> bool {
		let removed = REMOTE.write().remove(kind, plugin);
		if removed {
			Self::pub_inner_hi();
		}
		removed
	}

	pub fn subscriptions(kind: &str, remote: bool) -> Vec<Subscription> {
		let subscriptions = if remote { REMOTE.read() } else { LOCAL.read() };
		subscriptions.get(kind).into_iter().flatten().cloned().collect()
	}

	pub fn r#pub(body: Ember<'static>) -> Result<()> {
		let payload = body.with_receiver(BOOT.id);
		payload.try_flush()?;
		payload.emit();
		Ok(())
	}

	pub fn pub_to(receiver: Id, body: Ember<'static>) -> Result<()> {
		if receiver == BOOT.id {
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

	pub(crate) fn pub_inner_hi() {
		let subscriptions = REMOTE.read();
		let abilities = BOOT.remote_events.iter().chain(subscriptions.keys()).map(AsRef::as_ref);

		// FIXME: handle error
		Client::push(EmberHi::borrowed(abilities)).ok();
	}

	fn any_remote_own(kind: &str) -> bool {
		REMOTE.read().contains_key(kind)  // Own remote abilities
			|| PEERS.read().values().any(|p| p.able(kind))  // Remote peers' abilities
			|| BOOT.remote_events.contains(kind) // Own abilities from the command-line argument
	}
}

#[rustfmt::skip]
impl Pubsub {
	pub_after!(tab(idx: Id), (idx));

	pub_after!(cd(tab: Id, url: &UrlBuf), (tab, url));

	pub_after!(load(tab: Id, url: &UrlBuf, stage: &FolderStage), (tab, url, stage));

	pub_after!(patch(tab: Id, op: &FilesOp), (tab, op));

	pub_after!(hover(tab: Id, url: Option<&UrlBuf>), (tab, url));

	pub_after!(create(tab: Id, url: &UrlBuf), (tab, url));

	pub_after!(bulk_create<'a>(urls: impl Iterator<Item = Url<'a>> + Clone), (urls.clone()), (urls));

	pub_after!(rename(tab: Id, from: &UrlBuf, to: &UrlBuf), (tab, from, to));

	pub_after!(bulk_rename<'a>(changes: impl Iterator<Item = (Url<'a>, Url<'a>)> + Clone), (changes.clone()), (changes));

	pub_after!(@yank(cut: bool, files: &IndexSet<FileCov>), (cut, files));

	pub_after!(duplicate(items: Vec<EmberDuplicateItem>), (&items), (items));

	pub_after!(move(items: Vec<EmberMoveItem>), (&items), (items));

	pub_after!(trash(urls: Vec<UrlBuf>), (&urls), (urls));

	pub_after!(delete(urls: Vec<UrlBuf>), (&urls), (urls));

	pub_after!(download(urls: Vec<UrlBuf>), (&urls), (urls));

	pub_after!(input(r#type: &'static str, value: &str), (r#type, value));

	pub_after!(mount(), ());

	pub_after!(theme(), ());

	pub_after!(focus(lost: bool), (lost));
}
