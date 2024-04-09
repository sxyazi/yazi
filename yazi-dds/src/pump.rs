use std::time::Duration;

use tokio::{pin, select, sync::mpsc};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_shared::{fs::Url, RoCell};

use crate::{body::BodyMoveItem, Pubsub};

static MOVE_TX: RoCell<mpsc::UnboundedSender<BodyMoveItem>> = RoCell::new();
static DELETE_TX: RoCell<mpsc::UnboundedSender<Url>> = RoCell::new();

pub struct Pump;

impl Pump {
	#[inline]
	pub fn push_move(from: Url, to: Url) { MOVE_TX.send(BodyMoveItem { from, to }).ok(); }

	#[inline]
	pub fn push_delete(target: Url) { DELETE_TX.send(target).ok(); }

	pub(super) fn serve() {
		let (move_tx, move_rx) = mpsc::unbounded_channel();
		let (delete_tx, delete_rx) = mpsc::unbounded_channel();

		MOVE_TX.init(move_tx);
		DELETE_TX.init(delete_tx);

		tokio::spawn(async move {
			let move_rx =
				UnboundedReceiverStream::new(move_rx).chunks_timeout(1000, Duration::from_millis(500));
			let delete_rx =
				UnboundedReceiverStream::new(delete_rx).chunks_timeout(1000, Duration::from_millis(500));

			pin!(move_rx);
			pin!(delete_rx);

			loop {
				select! {
					Some(items) = move_rx.next() => Pubsub::pub_from_move(items),
					Some(targets) = delete_rx.next() => Pubsub::pub_from_delete(targets),
					else => break,
				}
			}
		});
	}

	pub(super) fn shutdown() {
		MOVE_TX.drop();
		DELETE_TX.drop();
	}
}
