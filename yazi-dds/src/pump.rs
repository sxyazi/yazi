use std::time::Duration;

use parking_lot::Mutex;
use tokio::{pin, select, sync::mpsc};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;
use yazi_shared::{RoCell, url::Url};

use crate::{Pubsub, body::BodyMoveItem};

static CT: RoCell<CancellationToken> = RoCell::new();
static MOVE_TX: Mutex<Option<mpsc::UnboundedSender<BodyMoveItem>>> = Mutex::new(None);
static TRASH_TX: Mutex<Option<mpsc::UnboundedSender<Url>>> = Mutex::new(None);
static DELETE_TX: Mutex<Option<mpsc::UnboundedSender<Url>>> = Mutex::new(None);

pub struct Pump;

impl Pump {
	#[inline]
	pub fn push_move(from: Url, to: Url) {
		if let Some(tx) = &*MOVE_TX.lock() {
			tx.send(BodyMoveItem { from, to }).ok();
		}
	}

	#[inline]
	pub fn push_trash(target: Url) {
		if let Some(tx) = &*TRASH_TX.lock() {
			tx.send(target).ok();
		}
	}

	#[inline]
	pub fn push_delete(target: Url) {
		if let Some(tx) = &*DELETE_TX.lock() {
			tx.send(target).ok();
		}
	}

	pub(super) fn serve() {
		let (move_tx, move_rx) = mpsc::unbounded_channel();
		let (trash_tx, trash_rx) = mpsc::unbounded_channel();
		let (delete_tx, delete_rx) = mpsc::unbounded_channel();

		CT.with(<_>::default);
		MOVE_TX.lock().replace(move_tx);
		TRASH_TX.lock().replace(trash_tx);
		DELETE_TX.lock().replace(delete_tx);

		tokio::spawn(async move {
			let move_rx =
				UnboundedReceiverStream::new(move_rx).chunks_timeout(1000, Duration::from_millis(500));
			let trash_rx =
				UnboundedReceiverStream::new(trash_rx).chunks_timeout(1000, Duration::from_millis(500));
			let delete_rx =
				UnboundedReceiverStream::new(delete_rx).chunks_timeout(1000, Duration::from_millis(500));

			pin!(move_rx);
			pin!(trash_rx);
			pin!(delete_rx);

			loop {
				select! {
					Some(items) = move_rx.next() => Pubsub::pub_from_move(items),
					Some(urls) = trash_rx.next() => Pubsub::pub_from_trash(urls),
					Some(urls) = delete_rx.next() => Pubsub::pub_from_delete(urls),
					else => {
						CT.cancel();
						break;
					},
				}
			}
		});
	}

	pub(super) async fn shutdown() {
		drop(MOVE_TX.lock().take());
		drop(TRASH_TX.lock().take());
		drop(DELETE_TX.lock().take());
		CT.cancelled().await;
	}
}
